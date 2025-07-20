# Contributing to LazyCelery

Thank you for your interest in contributing to LazyCelery! This document provides guidelines and instructions for contributing.

## Code of Conduct

By participating in this project, you agree to abide by our code of conduct: be respectful, inclusive, and constructive.

## How to Contribute

### Reporting Issues

- Check if the issue already exists
- Include steps to reproduce
- Include system information (OS, Rust version)
- Include relevant logs or error messages

### Submitting Pull Requests

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`mise run test`)
5. Run linting (`mise run lint`)
6. Commit with conventional commits (see below)
7. Push to your fork
8. Open a Pull Request

### Conventional Commits

We use [Conventional Commits](https://www.conventionalcommits.org/) for our commit messages:

- `feat:` New features
- `fix:` Bug fixes
- `docs:` Documentation changes
- `style:` Code style changes (formatting, etc)
- `refactor:` Code refactoring
- `perf:` Performance improvements
- `test:` Test additions or corrections
- `chore:` Maintenance tasks
- `ci:` CI/CD changes

Example: `feat: add AMQP broker support`

### Development Setup

1. Install Rust (1.70.0 or later)
2. Clone the repository
3. Install mise (task runner):
   ```bash
   ./scripts/install-mise.sh
   ```
4. Setup development environment:
   ```bash
   mise run setup
   ```

This will:
- Install required Rust components
- Install development tools
- Start Redis using Docker
- Prepare your environment for development

### Running Tests

```bash
# Run all tests
mise run test

# Run tests in watch mode
mise run test-watch

# Run with coverage
mise run coverage

# Run specific test
cargo test test_worker_creation
```

### Code Style

- Run `mise run fmt` to format code
- Run `mise run lint` to check for issues
- Run `mise run check` to verify both formatting and linting
- Follow Rust naming conventions
- Add documentation for public APIs

### Pre-commit Checklist

Run before committing:
```bash
mise run pre-commit
```

This will run formatting, linting, tests, and security audit.

### Pull Request Process

1. Update documentation if needed
2. Add tests for new functionality
3. Ensure CI passes
4. Request review from maintainers
5. Address review feedback

## Release Process

Releases are automated via GitHub Actions:

1. Create a version bump PR using the Version Bump workflow
2. Merge the PR
3. Create and push a tag: `git tag v1.2.3 && git push origin v1.2.3`
4. GitHub Actions will create the release

## Questions?

Feel free to open an issue for any questions!