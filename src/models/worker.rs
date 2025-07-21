use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worker {
    pub hostname: String,
    pub status: WorkerStatus,
    pub concurrency: u32,
    pub queues: Vec<String>,
    pub active_tasks: Vec<String>,
    pub processed: u64,
    pub failed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkerStatus {
    Online,
    Offline,
}

impl Worker {
    /// Basic constructor for creating a new Worker - kept for future API use
    #[allow(dead_code)]
    pub fn new(hostname: String) -> Self {
        Self {
            hostname,
            status: WorkerStatus::Offline,
            concurrency: 1,
            queues: Vec::new(),
            active_tasks: Vec::new(),
            processed: 0,
            failed: 0,
        }
    }

    pub fn utilization(&self) -> f32 {
        if self.concurrency == 0 {
            0.0
        } else {
            (self.active_tasks.len() as f32 / self.concurrency as f32) * 100.0
        }
    }
}
