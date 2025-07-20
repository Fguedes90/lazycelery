# LazyCelery MVP Specification

## Overview

LazyCelery is a terminal-based user interface (TUI) for monitoring and managing Celery workers and tasks, inspired by lazydocker and lazygit. Written in Rust for performance, it provides real-time monitoring and basic task management for Celery deployments.

## Goals

- **Simplify Celery Monitoring**: Intuitive, keyboard-driven interface for monitoring Celery
- **Real-time Updates**: Live updates of task execution and worker status
- **Minimal Dependencies**: Single binary that connects directly to Redis/RabbitMQ
- **Performance**: Fast and responsive UI

## MVP Features

### 1. Worker Monitoring
- List active workers with status (online/offline)
- Display worker configuration (concurrency, queues)
- Show basic statistics (processed, failed, active tasks)
- View worker logs

### 2. Queue Monitoring
- Display queues with message counts
- Show consumption rates
- Basic queue actions (purge with confirmation)

### 3. Task Management
- Real-time task list with status (pending/active/success/failure)
- Task details view (name, args, kwargs, result, traceback)
- Search and filter tasks
- Retry failed tasks
- Revoke running tasks

### 4. Basic Metrics
- Task success/failure rates
- Queue lengths over time
- Worker utilization
- Task execution duration

## UI Design

### Simple Layout
```
┌─────────────────────────────────────────────────────────────┐
│ LazyCelery v0.1.0          [W]orkers [Q]ueues [T]asks [?]  │
├─────────────────┬───────────────────────────────────────────┤
│ Workers (3)     │ Worker: celery@worker-1                   │
│ ● worker-1      │ Status: Online                            │
│ ● worker-2      │ Tasks: 2/4 active                         │
│ ○ worker-3      │ Processed: 1,523 (12 failed)              │
│                 │                                           │
│ Queues (2)      │ Recent Tasks:                             │
│ default   125   │ [SUCCESS] send_email                      │
│ priority   12   │ [ACTIVE]  process_image (00:02:15)        │
│                 │ [FAILED]  calculate_report                │
│                 │                                           │
│ [↑↓] Navigate   │ [Enter] Details  [r] Retry  [x] Revoke   │
└─────────────────┴───────────────────────────────────────────┘
│ > Task calculate_report[abc123] failed: ZeroDivisionError   │
└──────────────────────────────────────────────────────────────┘
```

### Navigation
- Arrow keys or vim-style (hjkl) for navigation
- Tab to switch between panels
- Enter for details, Esc to go back
- / for search
- q to quit, ? for help

## Technical Architecture

### Core Components

1. **Broker Connection**
   - Redis client using `redis-rs`
   - RabbitMQ client using `lapin`
   - Basic connection pooling

2. **Simple Data Models**
   ```rust
   pub struct Worker {
       hostname: String,
       status: WorkerStatus,
       concurrency: u32,
       queues: Vec<String>,
       active_tasks: Vec<String>,
       processed: u64,
       failed: u64,
   }

   pub struct Task {
       id: String,
       name: String,
       args: String,  // JSON string
       kwargs: String, // JSON string
       status: TaskStatus,
       worker: Option<String>,
       timestamp: DateTime<Utc>,
       result: Option<String>,
       traceback: Option<String>,
   }

   pub struct Queue {
       name: String,
       length: u64,
       consumers: u32,
   }
   ```

3. **TUI Framework**
   - `ratatui` for UI
   - `crossterm` for terminal control
   - `tokio` for async operations

4. **State Management**
   - Simple app state with Arc<Mutex<>>
   - Periodic refresh (configurable interval)

## Implementation Plan

### Phase 1: Foundation (Week 1-2)
- Project setup and basic broker connections
- Worker listing and status
- Basic TUI layout

### Phase 2: Core Features (Week 3-4)
- Queue monitoring
- Task listing with status
- Task details view
- Search functionality

### Phase 3: Actions & Polish (Week 5-6)
- Task retry/revoke
- Basic metrics
- Configuration file support
- Error handling and stability

## Configuration

### CLI
```bash
lazycelery --broker redis://localhost:6379/0
lazycelery --broker amqp://localhost:5672//
```

### Config File (Optional)
```toml
[broker]
url = "redis://localhost:6379/0"

[ui]
refresh_interval = 1000  # ms
theme = "dark"
```

## Dependencies

### Essential Crates Only
- `ratatui` - TUI framework
- `crossterm` - Terminal manipulation
- `tokio` - Async runtime
- `redis` - Redis client
- `lapin` - RabbitMQ client
- `serde` / `serde_json` - JSON handling
- `chrono` - Timestamps
- `clap` - CLI args

## Success Criteria

- Single binary under 10MB
- Sub-100ms UI response
- Works with 1000+ tasks
- Zero configuration for basic use
- Clear, intuitive interface