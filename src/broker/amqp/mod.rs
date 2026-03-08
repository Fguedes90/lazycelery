//! AMQP (RabbitMQ) broker implementation for Celery
//!
//! This module provides AMQP broker support using the lapin library.
//! It connects to RabbitMQ and consumes Celery events from the `celeryev` queue.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures_lite::stream::StreamExt;
use lapin::{
    options::{
        BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, ExchangeDeclareOptions,
        QueueBindOptions, QueueDeclareOptions, QueuePurgeOptions,
    },
    types::FieldTable,
    BasicProperties, Channel, Connection, ConnectionProperties, Consumer,
};
use serde_json::Value;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::broker::Broker;
use crate::error::BrokerError;
use crate::models::{Queue, Task, TaskStatus, Worker, WorkerStatus};

/// Celery event types we care about
#[derive(Debug, Clone)]
enum CeleryEventType {
    WorkerOnline,
    WorkerOffline,
    TaskStarted,
    TaskSuccess,
    TaskFailure,
    TaskRetry,
    TaskReceived,
    Unknown,
}

/// Parsed Celery event
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct CeleryEvent {
    event_type: CeleryEventType,
    timestamp: f64,
    hostname: Option<String>,
    task_id: Option<String>,
    task_name: Option<String>,
    result: Option<String>,
    traceback: Option<String>,
    exception: Option<String>,
    args: Option<String>,
    kwargs: Option<String>,
    retries: Option<u32>,
}

impl CeleryEvent {
    /// Parse a raw event JSON into a CeleryEvent
    fn parse(data: &[u8]) -> Result<Self, BrokerError> {
        let json: Value = serde_json::from_slice(data).map_err(|e| {
            BrokerError::OperationError(format!("Failed to parse event JSON: {e}"))
        })?;

        let event_type = match json.get("type").and_then(|v| v.as_str()) {
            Some("worker-online") => CeleryEventType::WorkerOnline,
            Some("worker-offline") => CeleryEventType::WorkerOffline,
            Some("task-started") => CeleryEventType::TaskStarted,
            Some("task-success") => CeleryEventType::TaskSuccess,
            Some("task-failure") => CeleryEventType::TaskFailure,
            Some("task-retry") => CeleryEventType::TaskRetry,
            Some("task-received") => CeleryEventType::TaskReceived,
            _ => CeleryEventType::Unknown,
        };

        let timestamp = json
            .get("timestamp")
            .and_then(|v| v.as_f64())
            .unwrap_or_else(|| Utc::now().timestamp_millis() as f64);

        let hostname = json
            .get("hostname")
            .and_then(|v| v.as_str())
            .map(String::from);
        let task_id = json.get("id").and_then(|v| v.as_str()).map(String::from);
        let task_name = json.get("name").and_then(|v| v.as_str()).map(String::from);
        let result = json
            .get("result")
            .and_then(|v| v.as_str())
            .map(String::from);
        let traceback = json
            .get("traceback")
            .and_then(|v| v.as_str())
            .map(String::from);
        let exception = json
            .get("exception")
            .and_then(|v| v.as_str())
            .map(String::from);
        let args = json.get("args").and_then(|v| v.as_str()).map(String::from);
        let kwargs = json
            .get("kwargs")
            .and_then(|v| v.as_str())
            .map(String::from);
        let retries = json
            .get("retries")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);

        Ok(Self {
            event_type,
            timestamp,
            hostname,
            task_id,
            task_name,
            result,
            traceback,
            exception,
            args,
            kwargs,
            retries,
        })
    }

    /// Convert to Worker model
    fn to_worker(&self) -> Option<Worker> {
        let hostname = self.hostname.clone()?;
        let status = match self.event_type {
            CeleryEventType::WorkerOnline => WorkerStatus::Online,
            CeleryEventType::WorkerOffline => WorkerStatus::Offline,
            _ => return None,
        };

        Some(Worker {
            hostname,
            status,
            concurrency: 1,
            queues: vec![],
            active_tasks: vec![],
            processed: 0,
            failed: 0,
        })
    }

    /// Convert to Task model
    fn to_task(&self) -> Option<Task> {
        let task_id = self.task_id.clone()?;
        let task_name = self
            .task_name
            .clone()
            .unwrap_or_else(|| "unknown".to_string());

        let status = match self.event_type {
            CeleryEventType::TaskReceived => TaskStatus::Pending,
            CeleryEventType::TaskStarted => TaskStatus::Active,
            CeleryEventType::TaskSuccess => TaskStatus::Success,
            CeleryEventType::TaskFailure => TaskStatus::Failure,
            CeleryEventType::TaskRetry => TaskStatus::Retry,
            _ => return None,
        };

        Some(Task {
            id: task_id,
            name: task_name,
            args: self.args.clone().unwrap_or_else(|| "[]".to_string()),
            kwargs: self.kwargs.clone().unwrap_or_else(|| "{}".to_string()),
            status,
            worker: self.hostname.clone(),
            timestamp: DateTime::from_timestamp_millis(self.timestamp as i64)
                .unwrap_or_else(Utc::now),
            result: self.result.clone(),
            traceback: self.traceback.clone(),
        })
    }
}

/// AMQP broker state that persists across method calls
#[derive(Clone)]
struct AmqpState {
    /// Known workers discovered from events
    workers: Arc<RwLock<Vec<Worker>>>,
    /// Known tasks discovered from events
    tasks: Arc<RwLock<Vec<Task>>>,
    /// Whether we're connected and listening
    connected: Arc<RwLock<bool>>,
}

impl AmqpState {
    fn new() -> Self {
        Self {
            workers: Arc::new(RwLock::new(Vec::new())),
            tasks: Arc::new(RwLock::new(Vec::new())),
            connected: Arc::new(RwLock::new(false)),
        }
    }
}

/// AMQP broker implementation for Celery
pub struct AmqpBroker {
    /// AMQP connection
    connection: Connection,
    /// AMQP channel for operations
    channel: Channel,
    /// Broker URL
    #[allow(dead_code)]
    url: String,
    /// Internal state
    state: AmqpState,
    /// Celery event consumer (kept alive)
    #[allow(dead_code)]
    consumer: Option<Consumer>,
}

impl AmqpBroker {
    /// Create a new AMQP broker
    pub async fn connect(url: &str) -> Result<Self, BrokerError> {
        info!(
            "Connecting to AMQP broker: {}",
            url.split('@').next_back().unwrap_or("hidden")
        );

        let connection = Connection::connect(url, ConnectionProperties::default())
            .await
            .map_err(|e| {
                error!("Failed to connect to AMQP: {}", e);
                BrokerError::ConnectionError(format!("AMQP connection failed: {e}"))
            })?;

        let channel = connection
            .create_channel()
            .await
            .map_err(|e| BrokerError::OperationError(format!("Failed to create channel: {e}")))?;

        // Declare the celeryev exchange if it doesn't exist
        let _ = channel
            .exchange_declare(
                "celeryev",
                lapin::ExchangeKind::Topic,
                ExchangeDeclareOptions {
                    passive: false,
                    durable: true,
                    auto_delete: false,
                    internal: false,
                    nowait: false,
                },
                FieldTable::default(),
            )
            .await;

        info!("AMQP broker connected successfully");

        let broker = Self {
            connection,
            channel,
            url: url.to_string(),
            state: AmqpState::new(),
            consumer: None,
        };

        // Start consuming events in the background
        let _ = broker.start_event_consumer().await;

        Ok(broker)
    }

    /// Start consuming Celery events in the background
    async fn start_event_consumer(&self) -> Result<(), BrokerError> {
        let channel = self.connection.create_channel().await.map_err(|e| {
            BrokerError::OperationError(format!("Failed to create event channel: {e}"))
        })?;

        // Declare the celeryev queue
        let _queue = channel
            .queue_declare(
                "celeryev",
                QueueDeclareOptions {
                    passive: false,
                    durable: true,
                    exclusive: false,
                    auto_delete: false,
                    nowait: false,
                },
                FieldTable::default(),
            )
            .await
            .map_err(|e| BrokerError::OperationError(format!("Failed to declare queue: {e}")))?;

        // Bind to the celeryev exchange
        channel
            .queue_bind(
                "celeryev",
                "celeryev",
                "*",
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|e| BrokerError::OperationError(format!("Failed to bind queue: {e}")))?;

        // Create consumer
        let mut consumer = channel
            .basic_consume(
                "celeryev",
                "lazycelery",
                BasicConsumeOptions {
                    no_local: false,
                    no_ack: false,
                    exclusive: false,
                    nowait: false,
                },
                FieldTable::default(),
            )
            .await
            .map_err(|e| BrokerError::OperationError(format!("Failed to start consumer: {e}")))?;

        let workers = self.state.workers.clone();
        let tasks = self.state.tasks.clone();
        let connected = self.state.connected.clone();

        // Mark as connected
        {
            let mut conn = connected.write().await;
            *conn = true;
        }

        // Spawn the event consumer task
        tokio::spawn(async move {
            info!("Starting Celery event consumer");
            while let Some(delivery_result) = consumer.next().await {
                match delivery_result {
                    Ok(delivery) => {
                        let data = delivery.data.clone();
                        if let Ok(event) = CeleryEvent::parse(&data) {
                            #[allow(clippy::single_match)]
                            match event.to_worker() {
                                Some(worker) => {
                                    let mut workers_guard = workers.write().await;
                                    // Update or add worker
                                    if let Some(existing) = workers_guard
                                        .iter_mut()
                                        .find(|w| w.hostname == worker.hostname)
                                    {
                                        *existing = worker;
                                    } else {
                                        workers_guard.push(worker);
                                    }
                                }
                                None => {}
                            }

                            #[allow(clippy::single_match)]
                            match event.to_task() {
                                Some(task) => {
                                    let mut tasks_guard = tasks.write().await;
                                    // Update or add task
                                    if let Some(existing) =
                                        tasks_guard.iter_mut().find(|t| t.id == task.id)
                                    {
                                        // Update existing task
                                        existing.status = task.status;
                                        existing.result =
                                            task.result.clone().or(existing.result.clone());
                                        existing.traceback =
                                            task.traceback.clone().or(existing.traceback.clone());
                                        if let Some(w) = task.worker {
                                            existing.worker = Some(w);
                                        }
                                    } else {
                                        tasks_guard.push(task);
                                    }
                                }
                                None => {}
                            }
                        }
                        let _ = delivery.ack(BasicAckOptions::default()).await;
                    }
                    Err(e) => {
                        warn!("Error receiving event: {}", e);
                    }
                }
            }
            info!("Celery event consumer stopped");
        });

        Ok(())
    }

    /// List all queues in the broker
    async fn list_queues(&self) -> Result<Vec<String>, BrokerError> {
        // In AMQP/RabbitMQ, listing queues requires the management plugin
        // For basic implementation, we return common Celery queue patterns
        // The actual queues will be discovered when workers connect

        let mut queues = vec!["celery".to_string()];

        // Try to declare common Celery queues passively to check existence
        // This is a workaround since queue_list() requires management plugin
        let celery_queues = ["celery", "celery@", "celeryd", "celeryd@"];

        for queue_name in celery_queues {
            #[allow(clippy::single_match)]
            match self
                .channel
                .queue_declare(
                    queue_name,
                    QueueDeclareOptions {
                        passive: true,
                        durable: true,
                        exclusive: false,
                        auto_delete: false,
                        nowait: false,
                    },
                    FieldTable::default(),
                )
                .await
            {
                Ok(_) => {
                    if !queues.contains(&queue_name.to_string()) {
                        queues.push(queue_name.to_string());
                    }
                }
                Err(_) => {}
            }
        }

        Ok(queues)
    }
}

#[async_trait]
impl Broker for AmqpBroker {
    async fn connect(url: &str) -> Result<Self, BrokerError>
    where
        Self: Sized,
    {
        Self::connect(url).await
    }

    async fn get_workers(&self) -> Result<Vec<Worker>, BrokerError> {
        // Return workers from our cached state
        let workers = self.state.workers.read().await;
        Ok(workers.clone())
    }

    async fn get_tasks(&self) -> Result<Vec<Task>, BrokerError> {
        // Return tasks from our cached state
        let tasks = self.state.tasks.read().await;
        Ok(tasks.clone())
    }

    async fn get_queues(&self) -> Result<Vec<Queue>, BrokerError> {
        let queue_names = self.list_queues().await?;

        let mut queues = Vec::new();
        for name in queue_names {
            // Get queue info using passive declare
            #[allow(clippy::single_match)]
            match self
                .channel
                .queue_declare(
                    &name,
                    QueueDeclareOptions {
                        passive: true,
                        durable: true,
                        exclusive: false,
                        auto_delete: false,
                        nowait: false,
                    },
                    FieldTable::default(),
                )
                .await
            {
                Ok(declaration) => {
                    queues.push(Queue {
                        name,
                        length: declaration.message_count() as u64,
                        consumers: declaration.consumer_count(),
                    });
                }
                Err(e) => {
                    debug!("Could not get info for queue: {}", e);
                }
            }
        }

        Ok(queues)
    }

    async fn retry_task(&self, task_id: &str) -> Result<(), BrokerError> {
        // To retry a task in Celery via AMQP, we need to republish it
        // This requires knowing the original task message
        // For now, return not implemented - would need task message storage

        // Find the task to get its details
        let tasks = self.state.tasks.read().await;
        if let Some(task) = tasks.iter().find(|t| t.id == task_id) {
            // Republish the task to the default queue
            let payload = serde_json::json!({
                "id": task_id,
                "name": task.name,
                "args": task.args,
                "kwargs": task.kwargs,
                "retries": 1,
            });

            self.channel
                .basic_publish(
                    "",
                    "celery",
                    BasicPublishOptions::default(),
                    payload.to_string().as_bytes(),
                    BasicProperties::default()
                        .with_content_type("application/json".into())
                        .with_correlation_id(task_id.into()),
                )
                .await
                .map_err(|e| BrokerError::OperationError(format!("Failed to retry task: {e}")))?;

            info!("Task {} requeued for retry", task_id);
            Ok(())
        } else {
            Err(BrokerError::OperationError(format!(
                "Task {} not found",
                task_id
            )))
        }
    }

    async fn revoke_task(&self, task_id: &str) -> Result<(), BrokerError> {
        // To revoke a task, we publish a 'revoke' message to the Celery control exchange
        let revoke_msg = serde_json::json!({
            "action": "revoke",
            "task_id": task_id,
            "terminate": false,
        });

        self.channel
            .basic_publish(
                "celery",
                "celeryctl",
                BasicPublishOptions::default(),
                revoke_msg.to_string().as_bytes(),
                BasicProperties::default()
                    .with_content_type("application/json".into())
                    .with_correlation_id(task_id.into()),
            )
            .await
            .map_err(|e| BrokerError::OperationError(format!("Failed to revoke task: {e}")))?;

        info!("Task {} revoked", task_id);
        Ok(())
    }

    async fn purge_queue(&self, queue_name: &str) -> Result<u64, BrokerError> {
        // Purge a queue by redeclaring it with purge option
        let queue = self
            .channel
            .queue_declare(
                queue_name,
                QueueDeclareOptions {
                    passive: false,
                    durable: true,
                    exclusive: false,
                    auto_delete: false,
                    nowait: false,
                },
                FieldTable::default(),
            )
            .await
            .map_err(|e| BrokerError::OperationError(format!("Failed to declare queue: {e}")))?;

        let message_count = queue.message_count();

        // Purge messages by consuming them all
        if message_count > 0 {
            // Create a new channel for purging
            let purge_channel = self.connection.create_channel().await.map_err(|e| {
                BrokerError::OperationError(format!("Failed to create channel: {e}"))
            })?;

            // Purge the queue - returns u32 directly
            let purged = purge_channel
                .queue_purge(queue_name, QueuePurgeOptions::default())
                .await
                .map_err(|e| {
                    BrokerError::OperationError(format!("Failed to purge queue: {e}"))
                })?;

            info!("Purged {} messages from queue {}", purged, queue_name);
            Ok(purged as u64)
        } else {
            Ok(0)
        }
    }
}
