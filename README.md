# LazyCelery

A terminal UI for monitoring and managing Celery workers and tasks, inspired by lazydocker and lazygit.

## Features

- Real-time worker monitoring
- Queue management with message counts
- Task listing with status tracking
- Search and filter capabilities
- Keyboard-driven interface

## Installation

```bash
cargo build --release
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

This is an MVP implementation. See `specs/` directory for planned features.
