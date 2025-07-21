use crate::broker::Broker;
use crate::models::{Queue, Task, Worker};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Workers,
    Queues,
    Tasks,
}

#[derive(Debug, Clone)]
pub enum PendingAction {
    PurgeQueue(String),
    RetryTask(String),
    RevokeTask(String),
}

pub struct AppState {
    // Data state
    pub workers: Vec<Worker>,
    pub tasks: Vec<Task>,
    pub queues: Vec<Queue>,

    // Navigation state
    pub selected_tab: Tab,
    pub selected_worker: usize,
    pub selected_task: usize,
    pub selected_queue: usize,

    // UI state
    pub should_quit: bool,
    pub show_help: bool,
    pub search_query: String,
    pub is_searching: bool,

    // Dialog state
    pub show_confirmation: bool,
    pub confirmation_message: String,
    pub pending_action: Option<PendingAction>,
    pub status_message: String,

    // Task details state
    pub show_task_details: bool,
    pub selected_task_details: Option<Task>,

    // Broker
    pub(crate) broker: Arc<Mutex<Box<dyn Broker>>>,
}

impl AppState {
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
            show_task_details: false,
            selected_task_details: None,
            broker: Arc::new(Mutex::new(broker)),
        }
    }

    // Tab navigation
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

    // Item selection
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

    // UI state management
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

    // Task filtering
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

    // Dialog management
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

    // Status message management
    pub fn set_status_message(&mut self, message: String) {
        self.status_message = message;
    }

    pub fn clear_status_message(&mut self) {
        self.status_message.clear();
    }

    // Task details management
    pub fn show_task_details(&mut self) {
        if !self.tasks.is_empty() && self.selected_tab == Tab::Tasks {
            let filtered_tasks = self.get_filtered_tasks();
            if self.selected_task < filtered_tasks.len() {
                let task = filtered_tasks[self.selected_task];
                self.selected_task_details = Some(task.clone());
                self.show_task_details = true;
            }
        }
    }

    pub fn hide_task_details(&mut self) {
        self.show_task_details = false;
        self.selected_task_details = None;
    }

    // Data validation after refresh
    pub fn validate_selections(&mut self) {
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
    }
}
