//! Result backend trait for storing task results
//!
//! This module provides a trait for result backends that store task results,
//! tracebacks, and metadata. This is separate from the broker which handles
//! message routing.

use async_trait::async_trait;

use crate::error::BrokerError;
use crate::models::Task;

/// Result backend trait for storing task results
///
/// Result backends store the results of tasks after they complete,
/// including success results, failure tracebacks, and timing information.
#[async_trait]
pub trait ResultBackend: Send + Sync {
    /// Get a task result by ID
    async fn get_task_result(&self, task_id: &str) -> Result<Option<Task>, BrokerError>;

    /// Connect to the result backend
    async fn connect(url: &str) -> Result<Self, BrokerError>
    where
        Self: Sized;
}
