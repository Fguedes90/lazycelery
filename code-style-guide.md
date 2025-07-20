# LazyCelery Code Style Guide

## Project Structure

```
lazycelery/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs           # Entry point
│   ├── app.rs            # Application state
│   ├── config.rs         # Configuration handling
│   ├── broker/           # Broker connections
│   │   ├── mod.rs
│   │   ├── redis.rs
│   │   └── amqp.rs
│   ├── models/           # Data structures
│   │   ├── mod.rs
│   │   ├── worker.rs
│   │   ├── task.rs
│   │   └── queue.rs
│   ├── ui/               # Terminal UI
│   │   ├── mod.rs
│   │   ├── widgets/
│   │   │   ├── mod.rs
│   │   │   ├── workers.rs
│   │   │   ├── tasks.rs
│   │   │   └── queues.rs
│   │   └── events.rs
│   └── utils/            # Utilities
│       ├── mod.rs
│       └── formatting.rs
└── examples/
    └── config.toml
```

## Dependencies (Cargo.toml)

```toml
[dependencies]
# TUI
ratatui = "0.26"
crossterm = "0.27"

# Async runtime
tokio = { version = "1.35", features = ["full"] }

# Broker clients
redis = { version = "0.24", features = ["tokio-comp"] }
lapin = "2.3"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Time handling
chrono = { version = "0.4", features = ["serde"] }

# CLI
clap = { version = "4.4", features = ["derive"] }

# Error handling
thiserror = "1.0"
anyhow = "1.0"
```

## Code Structure Rules

### Error Handling
```rust
// Define custom error types using thiserror
#[derive(Debug, thiserror::Error)]
pub enum BrokerError {
    #[error("Connection failed: {0}")]
    ConnectionError(String),
    #[error("Authentication failed")]
    AuthError,
}

// Always return Result<T, Error> for fallible operations
// Use anyhow::Result for main() and top-level functions
// Use specific error types for library code
```

### Data Models
```rust
// models/worker.rs
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkerStatus {
    Online,
    Offline,
}
```

### UI Widget Pattern
```rust
pub struct WorkerWidget {
    selected: usize,
}

impl WorkerWidget {
    pub fn new() -> Self {
        Self { selected: 0 }
    }
    
    pub fn render(&mut self, f: &mut Frame, area: Rect, workers: &[Worker]) {
        // Render implementation
    }
    
    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => self.previous(),
            KeyCode::Down => self.next(),
            _ => {}
        }
    }
}
```

### Async Pattern
```rust
// All broker operations must be async
impl RedisBroker {
    pub async fn connect(url: &str) -> Result<Self, BrokerError> {
        // Implementation
    }
    
    pub async fn get_workers(&self) -> Result<Vec<Worker>, BrokerError> {
        // Implementation
    }
}
```

### Application State
```rust
// Single source of truth for all application data
pub struct App {
    pub workers: Vec<Worker>,
    pub tasks: Vec<Task>,
    pub queues: Vec<Queue>,
    pub selected_tab: Tab,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            workers: Vec::new(),
            tasks: Vec::new(),
            queues: Vec::new(),
            selected_tab: Tab::Workers,
            should_quit: false,
        }
    }
}
```

## Key Implementation Notes

1. **State Management**: Use a single `App` struct for all state
2. **Broker Interface**: Define a trait for broker operations, implement for Redis/AMQP
3. **Event Loop**: Separate UI events from data updates
4. **Error Display**: Show errors in the status bar, don't panic
5. **Data Refresh**: Update data every second in a background task

## Testing Approach

Write tests for:
- Model serialization/deserialization
- Broker connection and data parsing
- Widget keyboard navigation
- Configuration loading

## Performance Rules

- Don't block the UI thread
- Batch broker requests when possible
- Limit UI refresh to 10 FPS
- Use pagination for large lists