use async_trait::async_trait;
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client};
use crate::broker::Broker;
use crate::models::{Worker, WorkerStatus, Task, TaskStatus, Queue};
use crate::error::BrokerError;
use chrono::Utc;
use serde_json::Value;

pub struct RedisBroker {
    _client: Client,
    connection: MultiplexedConnection,
}

impl RedisBroker {
    async fn parse_workers(&self) -> Result<Vec<Worker>, BrokerError> {
        let mut conn = self.connection.clone();
        
        // Get all worker keys
        let _worker_keys: Vec<String> = conn
            .keys("celery-task-meta-*")
            .await
            .map_err(|e| BrokerError::OperationError(e.to_string()))?;
        
        let mut workers = Vec::new();
        
        // For MVP, we'll create mock workers since Celery's worker
        // representation in Redis is complex
        workers.push(Worker {
            hostname: "worker-1".to_string(),
            status: WorkerStatus::Online,
            concurrency: 4,
            queues: vec!["default".to_string(), "priority".to_string()],
            active_tasks: vec![],
            processed: 1523,
            failed: 12,
        });
        
        Ok(workers)
    }
    
    async fn parse_tasks(&self) -> Result<Vec<Task>, BrokerError> {
        let mut conn = self.connection.clone();
        
        // Get task results from celery-task-meta-* keys
        let task_keys: Vec<String> = conn
            .keys("celery-task-meta-*")
            .await
            .map_err(|e| BrokerError::OperationError(e.to_string()))?;
        
        let mut tasks = Vec::new();
        
        for key in task_keys.iter().take(100) { // Limit to 100 for performance
            if let Ok(data) = conn.get::<_, String>(key).await {
                if let Ok(task_data) = serde_json::from_str::<Value>(&data) {
                    let task = Task {
                        id: key.strip_prefix("celery-task-meta-")
                            .unwrap_or("unknown")
                            .to_string(),
                        name: task_data["task"]
                            .as_str()
                            .unwrap_or("unknown")
                            .to_string(),
                        args: task_data["args"]
                            .to_string(),
                        kwargs: task_data["kwargs"]
                            .to_string(),
                        status: match task_data["status"].as_str() {
                            Some("SUCCESS") => TaskStatus::Success,
                            Some("FAILURE") => TaskStatus::Failure,
                            Some("PENDING") => TaskStatus::Pending,
                            Some("RETRY") => TaskStatus::Retry,
                            Some("REVOKED") => TaskStatus::Revoked,
                            _ => TaskStatus::Active,
                        },
                        worker: task_data["hostname"]
                            .as_str()
                            .map(|s| s.to_string()),
                        timestamp: Utc::now(), // Would parse from task data
                        result: task_data["result"]
                            .as_str()
                            .map(|s| s.to_string()),
                        traceback: task_data["traceback"]
                            .as_str()
                            .map(|s| s.to_string()),
                    };
                    tasks.push(task);
                }
            }
        }
        
        Ok(tasks)
    }
    
    async fn parse_queues(&self) -> Result<Vec<Queue>, BrokerError> {
        let mut conn = self.connection.clone();
        
        // Common Celery queue names
        let queue_names = vec!["celery", "default", "priority"];
        let mut queues = Vec::new();
        
        for name in queue_names {
            let length: u64 = conn
                .llen(name)
                .await
                .unwrap_or(0);
            
            queues.push(Queue {
                name: name.to_string(),
                length,
                consumers: 0, // Would need to parse from worker info
            });
        }
        
        Ok(queues)
    }
}

#[async_trait]
impl Broker for RedisBroker {
    async fn connect(url: &str) -> Result<Self, BrokerError> {
        let client = Client::open(url)
            .map_err(|e| BrokerError::InvalidUrl(e.to_string()))?;
        
        let connection = client
            .get_multiplexed_tokio_connection()
            .await
            .map_err(|e| BrokerError::ConnectionError(e.to_string()))?;
        
        Ok(Self { _client: client, connection })
    }
    
    async fn get_workers(&self) -> Result<Vec<Worker>, BrokerError> {
        self.parse_workers().await
    }
    
    async fn get_tasks(&self) -> Result<Vec<Task>, BrokerError> {
        self.parse_tasks().await
    }
    
    async fn get_queues(&self) -> Result<Vec<Queue>, BrokerError> {
        self.parse_queues().await
    }
    
    async fn retry_task(&self, _task_id: &str) -> Result<(), BrokerError> {
        // In a real implementation, this would publish a new task message
        // For MVP, we'll return not implemented
        Err(BrokerError::NotImplemented)
    }
    
    async fn revoke_task(&self, _task_id: &str) -> Result<(), BrokerError> {
        // In a real implementation, this would add to revoked tasks set
        // For MVP, we'll return not implemented
        Err(BrokerError::NotImplemented)
    }
}
