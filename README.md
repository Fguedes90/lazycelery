# LazyCelery

[![CI](https://github.com/fguedes90/lazycelery/workflows/CI/badge.svg)](https://github.com/fguedes90/lazycelery/actions/workflows/ci.yml)
[![Release](https://github.com/fguedes90/lazycelery/workflows/Release/badge.svg)](https://github.com/fguedes90/lazycelery/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/lazycelery.svg)](https://crates.io/crates/lazycelery)

A terminal UI for monitoring and managing Celery workers and tasks, inspired by lazydocker and lazygit.

## Features

- Real-time worker monitoring
- Queue management with message counts
- Task listing with status tracking
- Search and filter capabilities
- Keyboard-driven interface
- Interactive CLI configuration with subcommands
- Automatic configuration file management
- Helpful error messages and setup guidance

## Screenshots

### Main Dashboard - Workers View
![Workers View](screenshots/workers-view.png)

### Queue Management
![Queues View](screenshots/queues-view.png)

### Task Monitoring
![Tasks View](screenshots/tasks-view.png)

### Search Mode
![Search Mode](screenshots/search-mode.png)

### Help Screen
![Help Screen](screenshots/help-screen.png)

## Installation

Choose your preferred installation method:

### 🦀 Cargo (Rust package manager)

```bash
cargo install lazycelery
```

### 🍺 Homebrew (macOS/Linux)

```bash
brew tap Fguedes90/tap
brew install lazycelery
```

### 🪣 Scoop (Windows)

```bash
scoop bucket add lazycelery https://github.com/Fguedes90/scoop-bucket.git
scoop install lazycelery
```

### 📥 Binary Download

Download pre-built binaries from [GitHub Releases](https://github.com/Fguedes90/lazycelery/releases):

- **Linux x86_64**: `lazycelery-linux-x86_64.tar.gz`
- **macOS x86_64**: `lazycelery-macos-x86_64.tar.gz`  
- **macOS ARM64**: `lazycelery-macos-aarch64.tar.gz`
- **Windows x86_64**: `lazycelery-windows-x86_64.zip`

### 🔧 From Source

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

## Quick Start

### First Time Setup

```bash
# Run interactive setup
lazycelery init

# Or start with default Redis configuration
lazycelery --broker redis://localhost:6379/0
```

### Configuration Management

LazyCelery provides several subcommands to manage your configuration without editing files:

```bash
# Initialize configuration with interactive setup
lazycelery init

# Show current configuration
lazycelery config

# Update broker URL
lazycelery set-broker redis://localhost:6379/0

# Update refresh interval (milliseconds)
lazycelery set-refresh 2000
```

### Running LazyCelery

```bash
# Use configured settings
lazycelery

# Override broker URL
lazycelery --broker redis://localhost:6379/0

# Use custom config file
lazycelery --config ~/.config/lazycelery/config.toml
```

### Troubleshooting Connection Issues

If you encounter connection errors, LazyCelery provides helpful setup instructions:

1. **Start Redis** (choose one):
   ```bash
   # Docker
   docker run -d -p 6379:6379 redis
   
   # macOS
   brew services start redis
   
   # Linux
   sudo systemctl start redis
   ```

2. **Verify Redis is running**:
   ```bash
   redis-cli ping
   ```

3. **Run LazyCelery**:
   ```bash
   lazycelery --broker redis://localhost:6379/0
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

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
