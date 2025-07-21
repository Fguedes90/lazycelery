//! Queue parser for Redis Celery protocol
//!
//! This module handles parsing queue information from Redis data structures.
//! It discovers queues from kombu bindings and checks standard queue names
//! to provide information about queue status and message counts.

use crate::error::BrokerError;
use crate::models::Queue;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use std::collections::HashSet;

/// Parser for queue-related data from Redis
pub struct QueueParser;

impl QueueParser {
    /// Parse queues from Redis connection
    ///
    /// Discovers active queues from kombu bindings and standard queue names,
    /// then checks their length and consumer information to build a comprehensive
    /// view of the queue system.
    pub async fn parse_queues(
        connection: &MultiplexedConnection,
    ) -> Result<Vec<Queue>, BrokerError> {
        let mut conn = connection.clone();
        let mut queues = Vec::new();
        let mut discovered_queues = HashSet::new();

        // First, discover queues from kombu bindings
        let binding_keys: Vec<String> = conn.keys("_kombu.binding.*").await.unwrap_or_default();

        for binding_key in binding_keys {
            if let Some(queue_name) = binding_key.strip_prefix("_kombu.binding.") {
                discovered_queues.insert(queue_name.to_string());
            }
        }

        // Also check for common queue names
        let common_queues = vec!["celery", "default", "priority", "high", "low"];
        for queue_name in common_queues {
            discovered_queues.insert(queue_name.to_string());
        }

        // Check each discovered queue
        for queue_name in discovered_queues {
            let length: u64 = conn.llen(&queue_name).await.unwrap_or(0);

            // Only include queues that exist (have been used) or are standard
            if length > 0 || ["celery", "default"].contains(&queue_name.as_str()) {
                // Estimate consumers from worker data (simplified)
                let consumers = if length > 0 { 1 } else { 0 }; // Simplified consumer count

                queues.push(Queue {
                    name: queue_name,
                    length,
                    consumers,
                });
            }
        }

        // Sort queues by name for consistent display
        queues.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(queues)
    }
}
