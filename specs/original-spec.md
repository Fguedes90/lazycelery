# LazyCelery - Original Specification (Archive)

**Note**: This is the original specification kept for reference. See `mvp.md` for the current development target and `future-features.md` for planned enhancements.

---

# LazyCelery - Terminal UI for Celery

## Overview

LazyCelery is a terminal-based user interface (TUI) for monitoring and managing Celery workers and tasks, inspired by the intuitive interfaces of lazydocker and lazygit. Written in Rust for performance and reliability, it provides real-time monitoring, task management, and debugging capabilities for Celery deployments.

## Goals

- **Simplify Celery Monitoring**: Provide an intuitive, keyboard-driven interface for monitoring Celery workers, queues, and tasks
- **Real-time Updates**: Display live updates of task execution, worker status, and queue metrics
- **Minimal Dependencies**: Lightweight tool that connects directly to Redis/RabbitMQ brokers
- **Cross-platform**: Support Linux, macOS, and Windows environments
- **Performance**: Fast and responsive UI even with thousands of tasks

## Target Users

- DevOps engineers monitoring production Celery deployments
- Developers debugging Celery task execution
- System administrators managing worker pools
- Anyone who needs quick visibility into Celery operations

## Core Features

### 1. Worker Management
- List all active workers with status (online/offline/heartbeat)
- Display worker configuration (concurrency, pool type, queues)
- Show worker statistics (processed tasks, active tasks, reserved tasks)
- Start/stop/restart workers (if permissions allow)
- View worker logs in real-time
- Filter workers by hostname, queue, or status

### 2. Queue Monitoring
- Display all queues with message counts
- Show queue consumption rates
- Priority queue support
- Queue purging capabilities
- Message inspection (peek at messages without consuming)
- Queue routing visualization

### 3. Task Management
- Real-time task list with status (pending/active/success/failure/retry)
- Task details view (args, kwargs, result, traceback)
- Task timeline/history
- Retry failed tasks
- Revoke/terminate running tasks
- Task search and filtering (by name, status, worker, time range)
- Task result inspection

### 4. Performance Metrics
- Task execution time statistics
- Success/failure rates
- Queue backlog trends
- Worker utilization
- Memory usage per worker
- Broker connection health

### 5. Configuration & Settings
- Support for multiple broker connections
- Customizable refresh intervals
- Color themes (including colorblind-friendly options)
- Keyboard shortcut customization
- Export configurations

### 6. Debugging & Observability

#### Task Execution Tracing
- **Task Chain Visualization**: Display parent-child relationships and task dependencies
- **Execution Timeline**: Visual timeline showing when tasks started, retried, and completed
- **Call Stack View**: Show the complete call chain that triggered a task
- **Task Flow Diagram**: Interactive graph showing task execution flow
- **Breadcrumb Trail**: Track the sequence of events leading to task execution

#### Error Analysis
- **Error Inspector**: Detailed view of exceptions with full traceback
- **Error Patterns**: Identify recurring error patterns across tasks
- **Root Cause Analysis**: Trace errors back to their origin in task chains
- **Error Heatmap**: Visualize which tasks/workers are failing most frequently
- **Smart Error Grouping**: Group similar errors together for easier analysis

#### Advanced Debugging Tools
- **Task Replay**: Re-execute failed tasks with the same arguments for debugging
- **Breakpoint System**: Set conditional breakpoints on task execution
- **Variable Inspector**: Examine task arguments, kwargs, and intermediate results
- **Memory Profiler**: Track memory usage during task execution
- **Performance Profiler**: Identify bottlenecks in task execution

#### Distributed Tracing
- **OpenTelemetry Integration**: Full support for distributed tracing standards
- **Trace Context Propagation**: Follow requests across multiple services
- **Span Visualization**: See detailed timing for each operation within a task
- **Cross-Service Correlation**: Link Celery tasks with HTTP requests, database queries
- **Custom Instrumentation Points**: Add custom trace points in task code

#### Observability Features
- **Real-time Metrics Dashboard**: CPU, memory, network usage per worker/task
- **Anomaly Detection**: Alert on unusual patterns (sudden spike in failures, slow tasks)
- **Task Execution Heatmap**: Visualize task distribution across time and workers
- **Dependency Graph**: Show which tasks depend on external services
- **SLA Monitoring**: Track if tasks are meeting performance requirements

### 7. Task Relationship & Flow Analysis

#### Task Dependencies
- **Dependency Tree View**: Visualize parent-child task relationships
- **Canvas/Chain/Group Detection**: Automatically detect and display Celery primitives
- **Critical Path Analysis**: Identify the longest execution path in task chains
- **Circular Dependency Detection**: Warn about potential infinite loops

#### Execution Context
- **Request Tracing**: Link tasks to the original HTTP request or trigger
- **User Journey Tracking**: Follow a user's actions through multiple tasks
- **Business Transaction View**: Group related tasks by business operation
- **Event Sourcing Integration**: Show events that triggered task execution

## UI Design

### Layout (Inspired by lazydocker/lazygit)

#### Main Dashboard View
```
┌─────────────────────────────────────────────────────────────────┐
│ LazyCelery v0.1.0              [Q]ueues [W]orkers [T]asks [?]  │
├───────────────────┬─────────────────────────────────────────────┤
│ Workers (4)       │ Worker Details: celery@worker-1             │
│ ┌───────────────┐ │ ─────────────────────────────────────────── │
│ │▸ worker-1 ●   │ │ Status: Online                              │
│ │  worker-2 ●   │ │ Pool: prefork (4 processes)                 │
│ │  worker-3 ●   │ │ Queues: default, priority                   │
│ │  worker-4 ○   │ │ Active Tasks: 2/4                           │
│ └───────────────┘ │ Processed: 1,523                            │
│                   │ Failed: 12                                  │
│ Queues (3)        │ Success Rate: 99.2%                         │
│ ┌───────────────┐ │                                             │
│ │ default   125 │ ├─────────────────────────────────────────────┤
│ │ priority   12 │ │ Active Tasks                                │
│ │ celery      0 │ │ ─────────────────────────────────────────── │
│ └───────────────┘ │ ID      Task              Status  Duration  │
│                   │ abc123  send_email        ACTIVE  00:02:15  │
│ [Enter] Details   │ def456  process_image     ACTIVE  00:00:45  │
│ [r] Refresh       │                                             │
│ [/] Search        │ [Space] Select  [r] Retry  [x] Revoke      │
└───────────────────┴─────────────────────────────────────────────┘
│ Logs │ 2024-01-10 10:23:45 [INFO] Task send_email started    │
│      │ 2024-01-10 10:23:44 [INFO] Received task abc123      │
└──────┴──────────────────────────────────────────────────────┘
```

#### Debug View - Task Execution Flow
```
┌─────────────────────────────────────────────────────────────────┐
│ LazyCelery v0.1.0 - Debug Mode      [D]ebug [T]race [E]rrors   │
├───────────────────┬─────────────────────────────────────────────┤
│ Task Flow         │ Task Details: process_order[abc123]         │
│ ┌───────────────┐ │ ─────────────────────────────────────────── │
│ │ HTTP Request  │ │ Status: FAILED                              │
│ │      ↓        │ │ Started: 2024-01-10 10:23:45               │
│ │ validate_order│ │ Failed: 2024-01-10 10:23:47                │
│ │      ↓        │ │ Duration: 2.15s                             │
│ │ process_order │ │ Retries: 2/3                                │
│ │    ↓   ↓      │ │                                             │
│ │ email  charge │ │ Exception:                                  │
│ │         ↓     │ │ ─────────────────────────────────────────── │
│ │      FAILED ❌ │ │ ConnectionError: Payment gateway timeout    │
│ └───────────────┘ │                                             │
│                   │ Traceback:                                  │
│ Parent Chain:     │ File "tasks.py", line 45, in charge_card    │
│ ┌───────────────┐ │   response = gateway.charge(amount, card)   │
│ │ → HTTP /order │ │ File "gateway.py", line 123, in charge      │
│ │ → celery.chain│ │   raise ConnectionError("Timeout")          │
│ │ → process_task│ │                                             │
│ └───────────────┘ │ [i] Inspect Variables [r] Replay [t] Trace │
└───────────────────┴─────────────────────────────────────────────┘
│ Timeline │ 10:23:45 ──●────●────●────●──X  10:23:47        │
│          │   Start   Valid  Process Email Fail              │
└──────────┴──────────────────────────────────────────────────┘
```

#### Trace View - Distributed Tracing
```
┌─────────────────────────────────────────────────────────────────┐
│ LazyCelery v0.1.0 - Trace View     [S]pans [M]etrics [L]ogs    │
├─────────────────────────────────────────────────────────────────┤
│ Trace ID: 4bf92f3577b34da6a3ce929d0e0e4736                      │
│ Total Duration: 3.45s                                           │
│                                                                 │
│ Service    Operation              Duration   Status             │
│ ─────────────────────────────────────────────────────────────── │
│ ▼ api      POST /api/order        3.45s     ✓                  │
│   ▼ celery validate_order         0.12s     ✓                  │
│     └─ db  SELECT * FROM users    0.05s     ✓                  │
│   ▼ celery process_order          2.98s     ❌                 │
│     ├─ celery send_email          0.45s     ✓                  │
│     │  └─ smtp connect            0.38s     ✓                  │
│     └─ celery charge_card         2.15s     ❌ Timeout         │
│        └─ http payment.gateway    2.15s     ❌ 504             │
│                                                                 │
│ Span Details: charge_card                                       │
│ ─────────────────────────────────────────────────────────────── │
│ Tags:                                                           │
│   customer.id: 12345                                            │
│   payment.amount: 99.99                                         │
│   payment.currency: USD                                         │
│   error: true                                                   │
│   error.type: ConnectionError                                   │
└─────────────────────────────────────────────────────────────────┘
```

### Navigation
- **Vim-style keybindings**: j/k for up/down, h/l for left/right panels
- **Tab navigation**: Quick switch between Workers/Queues/Tasks views
- **Search mode**: / to search, n/N for next/previous
- **Context menus**: Right-click or Space for actions
- **Global shortcuts**: q to quit, ? for help, r to refresh

## Technical Architecture

### Core Components (Rust)

1. **Broker Connection Layer**
   - Redis client using `redis-rs`
   - RabbitMQ client using `lapin` 
   - Connection pooling and retry logic
   - Multi-broker support

2. **Data Models**
   ```rust
   pub struct Worker {
       hostname: String,
       status: WorkerStatus,
       pool: PoolType,
       concurrency: u32,
       queues: Vec<String>,
       active_tasks: Vec<Task>,
       stats: WorkerStats,
       system_metrics: SystemMetrics,
   }

   pub struct Task {
       id: String,
       name: String,
       args: Vec<serde_json::Value>,
       kwargs: HashMap<String, serde_json::Value>,
       status: TaskStatus,
       worker: Option<String>,
       timestamp: DateTime<Utc>,
       result: Option<TaskResult>,
       parent_id: Option<String>,
       children_ids: Vec<String>,
       trace_context: Option<TraceContext>,
       error_info: Option<ErrorInfo>,
       execution_timeline: Vec<TimelineEvent>,
   }

   pub struct Queue {
       name: String,
       messages: u64,
       consumers: u32,
       routing_key: Option<String>,
       message_rate: MessageRate,
   }

   pub struct TraceContext {
       trace_id: String,
       span_id: String,
       parent_span_id: Option<String>,
       baggage: HashMap<String, String>,
       start_time: DateTime<Utc>,
       duration: Duration,
       spans: Vec<Span>,
   }

   pub struct ErrorInfo {
       exception_type: String,
       message: String,
       traceback: Vec<TracebackFrame>,
       local_vars: HashMap<String, serde_json::Value>,
       system_state: SystemSnapshot,
       similar_errors: Vec<String>,
   }

   pub struct TaskFlow {
       root_task_id: String,
       nodes: HashMap<String, TaskNode>,
       edges: Vec<TaskEdge>,
       execution_order: Vec<String>,
       critical_path: Vec<String>,
   }
   ```

3. **TUI Framework**
   - Built on `ratatui` (successor to tui-rs)
   - Crossterm for terminal manipulation
   - Event-driven architecture
   - Async runtime with Tokio

4. **State Management**
   - Centralized app state with Arc<Mutex<AppState>>
   - Event bus for UI updates
   - Background tasks for data fetching

5. **Debugging & Observability Components**
   - **Trace Collector**: Aggregates distributed traces from multiple sources
   - **Error Analyzer**: Pattern matching and clustering for similar errors
   - **Task Graph Builder**: Constructs real-time task dependency graphs
   - **Metrics Aggregator**: Collects and calculates performance metrics
   - **Timeline Reconstructor**: Builds execution timelines from events
   - **Variable Inspector**: Safe serialization of task arguments and results

### Data Flow

```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│   Broker    │────▶│ Connection   │────▶│    State    │
│(Redis/AMQP) │     │   Manager    │     │   Manager   │
└─────────────┘     └──────────────┘     └─────────────┘
                                                 │
                    ┌──────────────┐             │
                    │   Terminal   │◀────────────┘
                    │      UI      │
                    └──────────────┘
                           │
                    ┌──────────────┐
                    │ User Input   │
                    │   Handler    │
                    └──────────────┘
```

## Implementation Plan

### Phase 1: Core Infrastructure (Weeks 1-2)
- Set up Rust project structure
- Implement broker connection layer
- Create basic data models
- Set up TUI framework with basic layout

### Phase 2: Worker & Queue Features (Weeks 3-4)
- Worker listing and details
- Queue monitoring
- Basic task display
- Real-time updates

### Phase 3: Task Management (Weeks 5-6)
- Task details view
- Task actions (retry, revoke)
- Search and filtering
- Task history

### Phase 4: Advanced Features (Weeks 7-8)
- Performance metrics
- Log streaming
- Configuration management
- Export capabilities

### Phase 5: Debugging & Observability (Weeks 9-10)
- Task flow visualization
- Error analysis and clustering
- Distributed tracing integration
- Variable inspection and replay
- Anomaly detection

### Phase 6: Polish & Testing (Weeks 11-12)
- Error handling
- Performance optimization
- Documentation
- Cross-platform testing

## Dependencies

### Rust Crates
- `ratatui` - Terminal UI framework
- `crossterm` - Terminal manipulation
- `tokio` - Async runtime
- `redis` - Redis client
- `lapin` - RabbitMQ client
- `serde` / `serde_json` - Serialization
- `chrono` - Date/time handling
- `clap` - CLI argument parsing
- `config` - Configuration management
- `tracing` - Logging framework
- `opentelemetry` - Distributed tracing
- `opentelemetry-otlp` - OTLP exporter
- `petgraph` - Graph algorithms for task flows
- `similar` - Text diffing for error comparison
- `backtrace` - Capture stack traces
- `sysinfo` - System metrics collection
- `dashmap` - Concurrent hashmap for metrics
- `regex` - Pattern matching for errors
- `tantivy` - Full-text search for logs/errors

### External Requirements
- Rust 1.70+
- Access to Celery broker (Redis/RabbitMQ)
- Terminal with 256-color support

## Configuration

### CLI Arguments
```bash
lazycelery --broker redis://localhost:6379/0
lazycelery --broker amqp://guest:guest@localhost:5672//
lazycelery --config ~/.config/lazycelery/config.toml
```

### Configuration File (TOML)
```toml
[broker]
url = "redis://localhost:6379/0"
timeout = 30
retry_attempts = 3

[ui]
theme = "dark"
refresh_interval = 1000  # milliseconds
show_timestamps = true

[keybindings]
quit = "q"
refresh = "r"
search = "/"
```

## Use Cases & Debugging Scenarios

### Scenario 1: Production Issue - Task Chain Failure
A user reports that their order isn't being processed. Using LazyCelery:
1. Search for the user's order ID across all tasks
2. View the task execution flow to see where it failed
3. Inspect the error details and local variables at failure point
4. Trace back through parent tasks to understand the context
5. Use task replay to reproduce the issue in a test environment

### Scenario 2: Performance Degradation
System is running slow during peak hours:
1. View the task execution heatmap to identify bottlenecks
2. Check worker utilization and memory usage
3. Analyze task duration trends over time
4. Identify tasks taking longer than SLA
5. Drill down into specific slow tasks to see distributed traces

### Scenario 3: Intermittent Errors
Random failures occurring across different workers:
1. Use error clustering to group similar failures
2. Identify patterns (specific workers, time of day, task types)
3. View system metrics at the time of failures
4. Correlate with external service health
5. Set up anomaly detection alerts for future occurrences

### Scenario 4: Complex Business Flow Analysis
Understanding how a multi-step business process executes:
1. View the complete task dependency graph
2. See the critical path through the workflow
3. Identify potential optimization opportunities
4. Track user journey from initial request to completion
5. Export flow diagram for documentation

## Security Considerations

- No storage of broker credentials in plain text
- Support for TLS connections
- Read-only mode by default
- Audit logging for destructive operations
- Configurable permissions for task actions
- Sanitization of sensitive data in debug views
- Role-based access control for debugging features

## Future Enhancements

- Flower API compatibility
- Task dependency visualization
- Multi-cluster support
- Plugin system for custom monitors
- HTTP API for integration
- Mobile-responsive web UI
- Celery Beat schedule monitoring
- Task result backend support
- Prometheus metrics export

## Success Metrics

- Sub-100ms UI response time
- Support for 10k+ tasks without performance degradation
- 90% reduction in time to diagnose Celery issues
- Zero-dependency installation (single binary)
- Cross-platform compatibility

## References

- [Celery Documentation](https://docs.celeryproject.org/)
- [lazydocker](https://github.com/jesseduffield/lazydocker)
- [lazygit](https://github.com/jesseduffield/lazygit)
- [Ratatui Documentation](https://ratatui.rs/)