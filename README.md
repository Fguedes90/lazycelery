# LazyCelery

[![CI](https://github.com/fguedes90/lazycelery/workflows/CI/badge.svg)](https://github.com/fguedes90/lazycelery/actions/workflows/ci.yml)
[![Release](https://github.com/fguedes90/lazycelery/workflows/Release/badge.svg)](https://github.com/fguedes90/lazycelery/releases)
[![Security Audit](https://github.com/fguedes90/lazycelery/workflows/Security%20Audit/badge.svg)](https://github.com/fguedes90/lazycelery/actions/workflows/security-audit.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/lazycelery.svg)](https://crates.io/crates/lazycelery)

A terminal UI for monitoring and managing Celery workers and tasks, inspired by lazydocker and lazygit.

## Features

- Real-time worker monitoring
- Queue management with message counts
- Task listing with status tracking
- Search and filter capabilities
- Keyboard-driven interface

## Installation

### Using Cargo

```bash
cargo install lazycelery
```

### From Source

```bash
# Clone the repository
git clone https://github.com/fguedes90/lazycelery.git
cd lazycelery

# Install mise (task runner)
./scripts/install-mise.sh

# Setup development environment
mise run setup

# Build release binary
mise run release
```

## Usage

```bash
# Connect to Redis broker
lazycelery --broker redis://localhost:6379/0

# Use configuration file
lazycelery --config ~/.config/lazycelery/config.toml
```

## Keyboard Shortcuts

- `Tab` - Switch between Workers/Queues/Tasks
- `↑/↓` or `j/k` - Navigate items
- `/` - Search mode
- `?` - Show help
- `q` - Quit

## Development

### Prerequisites

- Rust 1.70.0 or later
- Redis (for testing)
- [mise](https://mise.jdx.dev/) (task runner)

### Quick Start

```bash
# Install mise if you haven't already
./scripts/install-mise.sh

# Setup development environment
mise run setup

# Run with auto-reload
mise run dev

# Run tests in watch mode
mise run test-watch
```

### Available Tasks

```bash
mise tasks              # Show all available tasks
mise run build         # Build release binary
mise run dev           # Run with auto-reload
mise run test          # Run tests
mise run lint          # Run linter
mise run fmt           # Format code
mise run audit         # Security audit
mise run coverage      # Generate coverage report
mise run docs          # Generate documentation
```

### Pre-commit Checks

Before committing, run:

```bash
mise run pre-commit
```

This runs formatting, linting, tests, and security audit.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Specifications

See `specs/` directory for detailed specifications and planned features.
