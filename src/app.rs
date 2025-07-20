use crate::broker::Broker;
use crate::error::AppError;
use crate::models::{Queue, Task, Worker};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Workers,
    Queues,
    Tasks,
}

pub struct App {
    pub workers: Vec<Worker>,
    pub tasks: Vec<Task>,
    pub queues: Vec<Queue>,
    pub selected_tab: Tab,
    pub should_quit: bool,
    pub selected_worker: usize,
    pub selected_task: usize,
    pub selected_queue: usize,
    pub show_help: bool,
    pub search_query: String,
    pub is_searching: bool,
    pub show_confirmation: bool,
    pub confirmation_message: String,
    pub pending_action: Option<PendingAction>,
    pub status_message: String,
    broker: Arc<Mutex<Box<dyn Broker>>>,
}

#[derive(Debug, Clone)]
pub enum PendingAction {
    PurgeQueue(String),
    RetryTask(String),
    RevokeTask(String),
}

impl App {
    pub fn new(broker: Box<dyn Broker>) -> Self {
        Self {
            workers: Vec::new(),
            tasks: Vec::new(),
            queues: Vec::new(),
            selected_tab: Tab::Workers,
            should_quit: false,
            selected_worker: 0,
            selected_task: 0,
            selected_queue: 0,
            show_help: false,
            search_query: String::new(),
            is_searching: false,
            show_confirmation: false,
            confirmation_message: String::new(),
            pending_action: None,
            status_message: String::new(),
            broker: Arc::new(Mutex::new(broker)),
        }
    }

    pub async fn refresh_data(&mut self) -> Result<(), AppError> {
        let broker = self.broker.lock().await;

        // Fetch all data in parallel
        let (workers_result, tasks_result, queues_result) = tokio::join!(
            broker.get_workers(),
            broker.get_tasks(),
            broker.get_queues()
        );

        self.workers = workers_result?;
        self.tasks = tasks_result?;
        self.queues = queues_result?;

        // Ensure selection indices are valid
        if self.selected_worker >= self.workers.len() && !self.workers.is_empty() {
            self.selected_worker = self.workers.len() - 1;
        }
        if self.selected_task >= self.tasks.len() && !self.tasks.is_empty() {
            self.selected_task = self.tasks.len() - 1;
        }
        if self.selected_queue >= self.queues.len() && !self.queues.is_empty() {
            self.selected_queue = self.queues.len() - 1;
        }

        Ok(())
    }

    pub fn next_tab(&mut self) {
        self.selected_tab = match self.selected_tab {
            Tab::Workers => Tab::Queues,
            Tab::Queues => Tab::Tasks,
            Tab::Tasks => Tab::Workers,
        };
    }

    pub fn previous_tab(&mut self) {
        self.selected_tab = match self.selected_tab {
            Tab::Workers => Tab::Tasks,
            Tab::Queues => Tab::Workers,
            Tab::Tasks => Tab::Queues,
        };
    }

    pub fn select_next(&mut self) {
        match self.selected_tab {
            Tab::Workers => {
                if !self.workers.is_empty() {
                    self.selected_worker = (self.selected_worker + 1) % self.workers.len();
                }
            }
            Tab::Tasks => {
                let filtered_count = self.get_filtered_tasks().len();
                if filtered_count > 0 {
                    self.selected_task = (self.selected_task + 1) % filtered_count;
                }
            }
            Tab::Queues => {
                if !self.queues.is_empty() {
                    self.selected_queue = (self.selected_queue + 1) % self.queues.len();
                }
            }
        }
    }

    pub fn select_previous(&mut self) {
        match self.selected_tab {
            Tab::Workers => {
                if !self.workers.is_empty() {
                    self.selected_worker = if self.selected_worker == 0 {
                        self.workers.len() - 1
                    } else {
                        self.selected_worker - 1
                    };
                }
            }
            Tab::Tasks => {
                let filtered_count = self.get_filtered_tasks().len();
                if filtered_count > 0 {
                    self.selected_task = if self.selected_task == 0 {
                        filtered_count - 1
                    } else {
                        self.selected_task - 1
                    };
                }
            }
            Tab::Queues => {
                if !self.queues.is_empty() {
                    self.selected_queue = if self.selected_queue == 0 {
                        self.queues.len() - 1
                    } else {
                        self.selected_queue - 1
                    };
                }
            }
        }
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn start_search(&mut self) {
        self.is_searching = true;
        self.search_query.clear();
    }

    pub fn stop_search(&mut self) {
        self.is_searching = false;
        self.search_query.clear();
        // Reset selection when search is cleared
        if self.selected_tab == Tab::Tasks {
            self.selected_task = 0;
        }
    }

    pub fn get_filtered_tasks(&self) -> Vec<&Task> {
        if self.search_query.is_empty() {
            self.tasks.iter().collect()
        } else {
            self.tasks
                .iter()
                .filter(|task| {
                    task.name
                        .to_lowercase()
                        .contains(&self.search_query.to_lowercase())
                        || task
                            .id
                            .to_lowercase()
                            .contains(&self.search_query.to_lowercase())
                })
                .collect()
        }
    }

    pub fn show_confirmation_dialog(&mut self, message: String, action: PendingAction) {
        self.confirmation_message = message;
        self.pending_action = Some(action);
        self.show_confirmation = true;
    }

    pub fn hide_confirmation_dialog(&mut self) {
        self.show_confirmation = false;
        self.confirmation_message.clear();
        self.pending_action = None;
    }

    pub fn set_status_message(&mut self, message: String) {
        self.status_message = message;
    }

    pub fn clear_status_message(&mut self) {
        self.status_message.clear();
    }

    pub async fn execute_pending_action(&mut self) -> Result<(), AppError> {
        if let Some(action) = self.pending_action.take() {
            let message = {
                let broker = self.broker.lock().await;

                match &action {
                    PendingAction::PurgeQueue(queue_name) => {
                        match broker.purge_queue(queue_name).await {
                            Ok(count) => {
                                format!("Purged {count} messages from queue '{queue_name}'")
                            }
                            Err(e) => format!("Failed to purge queue '{queue_name}': {e}"),
                        }
                    }
                    PendingAction::RetryTask(task_id) => match broker.retry_task(task_id).await {
                        Ok(_) => format!("Task '{task_id}' marked for retry"),
                        Err(e) => format!("Failed to retry task '{task_id}': {e}"),
                    },
                    PendingAction::RevokeTask(task_id) => match broker.revoke_task(task_id).await {
                        Ok(_) => format!("Task '{task_id}' revoked"),
                        Err(e) => format!("Failed to revoke task '{task_id}': {e}"),
                    },
                }
            };

            self.set_status_message(message);
        }

        self.hide_confirmation_dialog();
        Ok(())
    }

    pub fn initiate_purge_queue(&mut self) {
        if !self.queues.is_empty() && self.selected_tab == Tab::Queues {
            let queue = &self.queues[self.selected_queue];
            let message = format!(
                "Are you sure you want to purge all {} messages from queue '{}'?",
                queue.length, queue.name
            );
            self.show_confirmation_dialog(message, PendingAction::PurgeQueue(queue.name.clone()));
        }
    }

    pub fn initiate_retry_task(&mut self) {
        if !self.tasks.is_empty() && self.selected_tab == Tab::Tasks {
            let filtered_tasks = self.get_filtered_tasks();
            if self.selected_task < filtered_tasks.len() {
                let task = filtered_tasks[self.selected_task];
                let message = format!("Are you sure you want to retry task '{}'?", task.id);
                self.show_confirmation_dialog(message, PendingAction::RetryTask(task.id.clone()));
            }
        }
    }

    pub fn initiate_revoke_task(&mut self) {
        if !self.tasks.is_empty() && self.selected_tab == Tab::Tasks {
            let filtered_tasks = self.get_filtered_tasks();
            if self.selected_task < filtered_tasks.len() {
                let task = filtered_tasks[self.selected_task];
                let message = format!("Are you sure you want to revoke task '{}'?", task.id);
                self.show_confirmation_dialog(message, PendingAction::RevokeTask(task.id.clone()));
            }
        }
    }
}
