use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub args: String,   // JSON string
    pub kwargs: String, // JSON string
    pub status: TaskStatus,
    pub worker: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub result: Option<String>,
    pub traceback: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Active,
    Success,
    Failure,
    Retry,
    Revoked,
}

impl Task {
    /// Basic constructor for creating a new Task - kept for future API use
    #[allow(dead_code)]
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            args: "[]".to_string(),
            kwargs: "{}".to_string(),
            status: TaskStatus::Pending,
            worker: None,
            timestamp: Utc::now(),
            result: None,
            traceback: None,
        }
    }

    pub fn duration_since(&self, now: DateTime<Utc>) -> chrono::Duration {
        now - self.timestamp
    }
}
