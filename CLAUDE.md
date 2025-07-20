# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

LazyCelery is a terminal UI for monitoring and managing Celery workers and tasks, inspired by lazydocker/lazygit. This is a Rust project with a fully functional architecture and UI framework. Currently implementing real Celery protocol integration to replace mock data systems.

## Development Commands

```bash
# Core development commands using mise:
mise run dev                            # Run with auto-reload (auto-starts Redis)
mise run test                           # Run all tests
mise run test-watch                     # Run tests in watch mode
mise run fmt                            # Format code
mise run lint                           # Lint code (clippy)
mise run audit                          # Security audit
mise run pre-commit                     # Run all checks before committing

# Setup and environment:
mise run setup                          # Setup development environment
mise run redis-start                    # Start Redis server via Docker
mise run redis-stop                     # Stop Redis server

# Specific commands for development:
cargo test --test integration           # Run specific test file
cargo test worker::tests               # Run specific module tests
cargo run -- --broker redis://localhost:6379/0  # Run with specific broker
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
- `broker/`: Async broker clients implementing common `Broker` trait (Redis fully functional, AMQP placeholder)
- `models/`: Complete data structures (Worker, Task, Queue) with serde serialization
- `ui/widgets/`: Fully implemented widgets (workers.rs, tasks.rs, queues.rs) with navigation and search
- `ui/events.rs`: Keyboard event handling including search mode and vim-style navigation
- `app.rs`: Central state management with tab navigation, selection, and search functionality
- `main.rs`: Complete CLI parsing, async tokio runtime, and event loop coordination
- `config.rs`: TOML configuration support with broker connection settings
- `error.rs`: Custom error types using `thiserror` for broker and application errors

## Current Implementation Status

**Fully Implemented (100%):**
- Complete terminal UI with workers/queues/tasks widgets and navigation
- Application architecture with centralized state management
- Configuration system with TOML support
- Error handling with custom error types
- Complete data models with serialization

**Partially Implemented (75%):**
- Redis broker client (connects and fetches basic data, but uses mock worker data)
- Task actions (retry/revoke exist in trait but return `NotImplemented`)

**Not Implemented (0%):**
- AMQP/RabbitMQ broker client (placeholder only)
- Real Celery protocol integration for worker discovery and task actions

## Current Development Focus

The project is on branch `feature/mvp-core-monitoring` implementing real Celery protocol integration. Priority tasks:
1. Replace mock worker data with real Celery worker parsing from Redis
2. Implement task retry/revoke functionality 
3. Add AMQP broker support
4. Enhance Redis parsing for actual Celery queue structures

## Testing Strategy

Comprehensive test suite exists for:
- Model serialization/deserialization
- UI widget state management and navigation
- Configuration loading from TOML files
- Error handling across all modules
- Basic broker functionality (with mocked responses)

**Test execution:**
- `mise run test` - Run all tests
- `mise run test-watch` - Continuous testing during development
- `cargo test --test integration` - Integration tests specifically
- `cargo test broker::tests::redis` - Redis broker tests

**Missing test coverage:**
- Real Celery protocol integration tests
- AMQP broker implementation tests
- End-to-end tests with actual Celery workers

## Performance Considerations

Current optimizations in place:
- 10 FPS UI refresh limit to reduce CPU usage
- Pagination for task lists over 100 items
- Async broker operations with proper error handling
- Efficient terminal rendering with ratatui

**Areas for future optimization:**
- Connection pooling for high-volume Redis operations
- Caching worker/queue data between refreshes
- Batching Redis requests to reduce network overhead