# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

LazyCelery is a terminal UI for monitoring and managing Celery workers and tasks, inspired by lazydocker/lazygit. This is a Rust project currently in the specification phase - no implementation exists yet.

## Development Commands

```bash
# Once project is initialized with cargo init:
cargo build                              # Build debug version
cargo build --release                    # Build optimized release
cargo run -- --broker redis://localhost:6379/0    # Run with Redis broker
cargo test                              # Run all tests
cargo fmt                               # Format code
cargo clippy                            # Lint code
```

## Architecture Overview

### Core Design Principles
1. **Single App State**: All application state lives in `src/app.rs` in the `App` struct
2. **Async Broker Operations**: All broker interactions use async/await with Tokio
3. **Trait-Based Broker Interface**: Common trait for Redis/AMQP implementations
4. **Widget-Based UI**: Each UI component is a separate widget with render() and handle_key()

### Data Flow Architecture
```
Broker (Redis/AMQP) → Async Broker Client → App State → UI Widgets → Terminal
                           ↑                     ↓
                           └── Background refresh task (1 second interval)
```

### Key Architectural Decisions
- **Error Handling**: Custom error types with `thiserror` for broker operations, `anyhow::Result` for main()
- **State Updates**: Background task updates data every second, UI thread only handles rendering
- **Event Loop**: Separate UI event handling from data updates to prevent blocking
- **UI Refresh**: Limited to 10 FPS to reduce CPU usage

### Module Responsibilities
- `broker/`: Implements async clients for Redis and AMQP with common trait
- `models/`: Simple data structures (Worker, Task, Queue) with serde serialization
- `ui/widgets/`: Individual UI components that render specific data types
- `app.rs`: Central state management and coordination
- `main.rs`: CLI parsing, tokio runtime setup, event loop

## Implementation Order

When implementing from scratch:
1. Create Cargo.toml with dependencies from code-style-guide.md
2. Implement models (Worker, Task, Queue structs)
3. Create broker trait and Redis implementation
4. Build basic TUI with worker listing
5. Add queue and task views
6. Implement task actions (retry, revoke)

## Testing Strategy

Focus tests on:
- Broker data parsing (mocking Redis/AMQP responses)
- Model serialization/deserialization
- Widget state management (navigation, selection)
- Configuration loading from TOML files

## Performance Considerations

- Batch broker requests to reduce network overhead
- Use pagination for task lists over 100 items
- Cache worker/queue data between refreshes
- Avoid cloning large Vec<Task> collections