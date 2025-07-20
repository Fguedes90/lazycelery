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

# Git hooks setup (run once after cloning):
git config core.hooksPath .githooks     # Enable pre-commit quality checks

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
- **Redis broker client with real Celery protocol integration**
- **Task retry/revoke functionality implemented**
- **Queue purge operations with confirmation dialogs**
- **Comprehensive CI/CD pipeline with automated releases**
- **Performance optimizations and caching**
- **Pre-commit hooks and quality gates**

**Partially Implemented (75%):**
- AMQP/RabbitMQ broker client (placeholder structure exists, needs real implementation)
- Advanced queue management features (basic operations work)

**Not Implemented (0%):**
- Real-time worker monitoring and stats collection
- Advanced task scheduling and workflow management
- Plugin system for custom brokers

## Current Development Focus

The project has successfully completed the MVP phase for Redis integration. Current status:
- **Branch**: `feature/mvp-core-monitoring` (ready for merge)
- **Redis Integration**: ✅ Complete with real Celery protocol parsing
- **Task Management**: ✅ Retry, revoke, and queue purge operations
- **CI/CD**: ✅ Fully automated releases and quality checks
- **Performance**: ✅ Optimized with aggressive caching

**Next priorities for future development:**
1. Complete AMQP/RabbitMQ broker implementation
2. Add real-time worker monitoring and health checks
3. Implement advanced task filtering and search
4. Add support for task routing and scheduling
5. Create plugin architecture for custom brokers

## Testing Strategy

**Comprehensive test suite exists for:**
- Model serialization/deserialization (complete)
- UI widget state management and navigation (complete)
- Configuration loading from TOML files (complete)
- Error handling across all modules (complete)
- **Redis broker functionality with real Celery protocol parsing (complete)**
- **Task operations (retry, revoke, queue purge) (complete)**
- **Integration tests with Redis service containers (complete)**
- **Cross-platform CI testing (Linux, macOS, Windows) (complete)**

**Test execution:**
- `mise run test` - Run all tests (enforced by pre-commit hooks)
- `mise run test-watch` - Continuous testing during development
- `cargo test --test integration` - Integration tests specifically
- `cargo test broker::tests::redis` - Redis broker tests
- **`cargo test --lib --bins` - Unit tests only (used in CI)**

**Test Coverage Status:**
- ✅ **Redis Celery protocol integration tests (100%)**
- ✅ **Task action tests with real Redis operations (100%)**
- ✅ **UI interaction and navigation tests (100%)**
- ✅ **Configuration and error handling tests (100%)**
- ⚠️  **AMQP broker implementation tests (placeholder only)**
- ⚠️  **End-to-end tests with actual Celery workers (manual testing only)**

**CI/CD Testing:**
- All tests run automatically on every PR
- Redis integration tests run with Docker service containers
- One integration test temporarily disabled due to CI interference issues
- Security audit runs on all dependencies
- Cross-platform testing ensures compatibility

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

## Release Management and CI/CD

### 🚨 CRITICAL: Automated Release System

This project has **100% AUTOMATED RELEASES** configured. Any push to `main` branch will automatically trigger a release if there are new commits since the last tag.

### Release Process (FULLY AUTOMATED)

1. **Automatic Trigger**: When PR is merged to `main` branch
2. **Smart Detection**: Analyzes commits to determine release type:
   - `feat:` or `feature:` → **MINOR** release (0.2.0 → 0.3.0)
   - `feat!:` or `BREAKING CHANGE` → **MAJOR** release (0.2.0 → 1.0.0)
   - Any other commit → **PATCH** release (0.2.0 → 0.2.1)
3. **Automatic Actions**:
   - Auto-bump version in Cargo.toml
   - Generate updated changelog using git-cliff
   - Create commit with `[skip ci]` to avoid loops
   - Create and push git tag
   - Publish to crates.io automatically
   - Create GitHub release with cross-platform binaries

### 🛡️ Release Safety Rules

**NEVER manually edit these without understanding the impact:**
- `Cargo.toml` version field (managed by automated releases)
- Git tags (created automatically by release workflow)
- CHANGELOG.md (generated automatically by git-cliff)

**To SKIP automatic release, use one of these in commit message:**
- `[skip ci]` - Skip all CI including release
- `chore:` prefix - Maintenance commits don't trigger releases
- `docs:` prefix - Documentation-only changes don't trigger releases

### 🔄 Workflow Files (DO NOT MODIFY WITHOUT REVIEW)

- `.github/workflows/ci.yml` - Main CI pipeline with optimized caching
- `.github/workflows/release.yml` - Automated release pipeline
- `.githooks/pre-commit` - Local quality gates before commits

### Conventional Commit Format (REQUIRED)

Use conventional commit format to ensure proper release type detection:

```bash
# Examples that trigger releases:
git commit -m "feat(broker): add AMQP support"           # → minor release
git commit -m "fix(ui): resolve task display issue"     # → patch release  
git commit -m "feat!: change broker interface"          # → major release

# Examples that DON'T trigger releases:
git commit -m "docs: update README [skip ci]"           # → no release
git commit -m "chore: cleanup code formatting"          # → no release
git commit -m "test: add more integration tests"        # → no release
```

### Quality Gates (ENFORCED)

All commits must pass:
- Code formatting (`cargo fmt`)
- Linting (`cargo clippy` with zero warnings)
- All tests (unit + integration, excluding Redis tests in CI)
- Security audit (`cargo audit`)

**Pre-commit hooks automatically enforce these locally.**

### Version Management

- Current version: **0.2.0** (in Cargo.toml)
- Versions are managed automatically by GitHub Actions
- Manual version bumps will be overridden by automated releases
- Use semantic versioning (semver) strictly

### Emergency Release Controls

If you need to emergency stop releases:
1. Add `[skip ci]` to all commits
2. Or temporarily disable the release workflow in GitHub Actions
3. Or create commits with `chore:` prefix only

### Configuration Secrets

The following repository secrets are required for automated releases:
- `CARGO_REGISTRY_TOKEN` - Token for publishing to crates.io (must be configured)

### Monitoring Releases

- Check GitHub Actions tab for release status
- Monitor crates.io for successful publications: https://crates.io/crates/lazycelery
- GitHub Releases page shows all published versions with binaries

## Claude Code Guidelines

### 🚨 CRITICAL: Release Management

**BEFORE making ANY changes to this repository:**

1. **Understand the automated release system** - This project auto-releases on every push to `main`
2. **Use conventional commits** - Your commit messages determine the release type
3. **Never manually edit versions** - The Cargo.toml version is managed automatically
4. **Test thoroughly** - All changes are automatically published to crates.io
5. **Use `[skip ci]` when appropriate** - For documentation or non-functional changes

### Preferred Commit Message Patterns

```bash
# Use these patterns for commits:
feat(module): add new functionality        # → triggers MINOR release
fix(module): resolve specific issue        # → triggers PATCH release  
feat!(module): breaking change            # → triggers MAJOR release
docs: update documentation [skip ci]      # → no release
chore: maintenance tasks [skip ci]        # → no release
test: add more test coverage              # → no release
```

### When Working on Features

1. **Always work on feature branches** - Never commit directly to `main`
2. **Run `mise run pre-commit`** before committing - Ensures quality
3. **Test locally first** - Use `mise run test` to verify changes
4. **Use descriptive commit messages** - They become part of changelog
5. **Consider release impact** - Your commits will trigger automatic releases

### Safe Operations

**These operations are SAFE and won't trigger releases:**
- Creating feature branches
- Making commits on feature branches  
- Running tests and development commands
- Editing documentation with `[skip ci]`
- Using `chore:` or `docs:` commit prefixes

**These operations WILL trigger releases:**
- Merging to `main` branch
- Any commit to `main` with `feat:` or `fix:` prefix
- Direct pushes to `main` (should be avoided)

### Emergency Procedures

If you accidentally trigger an unwanted release:
1. **Don't panic** - Releases are versioned and can be yanked from crates.io if needed
2. **Check GitHub Actions** - See if you can cancel the workflow before it publishes
3. **Contact the maintainer** - If a bad release was published
4. **Learn from it** - Use `[skip ci]` or `chore:` prefixes for non-functional changes

### Development Workflow

1. **Create feature branch**: `git checkout -b feature/your-feature`
2. **Make changes with proper testing**: `mise run test` 
3. **Use pre-commit checks**: `mise run pre-commit`
4. **Commit with conventional format**: `git commit -m "feat: your change"`
5. **Push and create PR**: Let CI validate before merge
6. **After PR merge**: Automatic release will be triggered

### File Modification Guidelines

**Modify freely:**
- `src/` directory (application code)
- `tests/` directory (test files)
- Documentation files (with `[skip ci]`)
- Configuration files (examples/, configs/)

**Modify with caution:**
- `Cargo.toml` dependencies (test thoroughly)
- `.github/workflows/` (can break CI/CD)
- `.githooks/` (can break quality gates)

**NEVER modify without explicit permission:**
- `Cargo.toml` version field (auto-managed)
- Git tags (auto-created)
- `CHANGELOG.md` (auto-generated)

This guidance ensures smooth collaboration while protecting the automated release system.