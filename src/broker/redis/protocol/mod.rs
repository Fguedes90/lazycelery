//! Redis protocol parsing modules
//!
//! This module contains parsers for different Celery protocol data types.
//! Each parser is responsible for parsing a specific type of data from Redis.

mod queue_parser;
mod task_parser;
mod worker_parser;

pub use queue_parser::QueueParser;
pub use task_parser::TaskParser;
pub use worker_parser::WorkerParser;

// Re-export the main ProtocolParser for backward compatibility
use crate::error::BrokerError;
use crate::models::{Queue, Task, Worker};
use redis::aio::MultiplexedConnection;

/// Main protocol parser that delegates to specialized parsers
pub struct ProtocolParser;

impl ProtocolParser {
    /// Parse workers from Redis connection
    pub async fn parse_workers(
        connection: &MultiplexedConnection,
    ) -> Result<Vec<Worker>, BrokerError> {
        WorkerParser::parse_workers(connection).await
    }

    /// Parse tasks from Redis connection
    pub async fn parse_tasks(connection: &MultiplexedConnection) -> Result<Vec<Task>, BrokerError> {
        TaskParser::parse_tasks(connection).await
    }

    /// Parse queues from Redis connection
    pub async fn parse_queues(
        connection: &MultiplexedConnection,
    ) -> Result<Vec<Queue>, BrokerError> {
        QueueParser::parse_queues(connection).await
    }
}
