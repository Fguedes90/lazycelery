## [0.7.2] - 2025-08-04

### 🐛 Bug Fixes

- Add rustfmt and clippy components to crates.io publish workflow
- Clarify Cargo installation method in README

### ⚙️ Miscellaneous Tasks

- Add success message to rust components installation
## [0.6.0] - 2025-08-04

### 🚀 Features

- Add CLI subcommands for improved configuration management
## [0.5.0] - 2025-08-03

### 🚀 Features

- Improve onboarding experience with better error messages and auto-config
## [0.4.5] - 2025-08-03

### 🐛 Bug Fixes

- Use proper shell syntax for Windows builds
- Use proper shell syntax for Windows builds
## [0.4.4] - 2025-08-03

### 🐛 Bug Fixes

- Prevent Prepare Release job from running on tag workflows
## [0.4.3] - 2025-08-03

### 🐛 Bug Fixes

- Add desktop.ini to gitignore for Windows compatibility
## [0.4.2] - 2025-08-03

### 🐛 Bug Fixes

- Correct workflow release logic to handle version bump scenarios
- Remove complex dependencies from release workflow
- Correct YAML syntax error in release workflow
- Add debugging category to package metadata
- Add write permissions to release workflow

### 💼 Other

- *(deps)* Bump base64 from 0.21.7 to 0.22.1
- *(deps)* Bump softprops/action-gh-release from 1 to 2
- *(deps)* Bump tokio from 1.46.1 to 1.47.0

### 🚜 Refactor

- Simplify release workflow logic for better reliability
## [0.4.1] - 2025-07-21

### 🐛 Bug Fixes

- Correct pre-commit hook configuration and resolve compilation issues
- Add comprehensive behavioral test coverage for untested modules
- Bump version to 0.4.1 for comprehensive test coverage release

### 🚜 Refactor

- Complete architectural refactoring for better modularity and maintainability
- Implement comprehensive code quality improvements

### ⚙️ Miscellaneous Tasks

- Tidy up project by removing unnecessary files
- Comprehensive project tidy and cleanup
- Enable tracking of test files in git
## [0.4.0] - 2025-07-20

### 🚀 Features

- *(ui)* Implement comprehensive task details modal
## [0.3.0] - 2025-07-20

### 🚀 Features

- Implement LazyCelery - Terminal UI for Celery monitoring
- Add Docker support and update project metadata
- Add comprehensive test suite for critical components
- Add CI/CD workflows and GitHub configuration
- Add professional terminal UI screenshots
- Add comprehensive project roadmap
- Complete MVP core actions with queue purge and confirmation dialogs
- Enhance CI/CD workflows with automated releases
- Add pre-commit hooks for code quality checks
- *(release)* Configure automated crates.io publishing
- Configure 100% automatic releases on PR merge
- Configure complete multi-platform package manager automation
- Complete MVP Core Actions v0.2.0 - Queue Purge & Confirmation Dialogs

### 🐛 Bug Fixes

- Resolve clippy warnings and improve code quality
- Resolve mise-action configuration issues
- Resolve CI/CD workflow failures
- Use separate Redis DB for integration tests to avoid CI conflicts
- Use unique task IDs in integration test to avoid interference
- Improve test assertion to focus on our specific test tasks
- Install cargo-audit before running security audit
- *(ci)* Remove unsupported --locked flag from cargo audit command

### 💼 Other

- Migrate from Makefile to mise task runner
- Add debug output to integration test to troubleshoot CI failure
- Add timing delay to ensure data persistence in CI environment

### 🚜 Refactor

- Rename master branch to main

### 📚 Documentation

- Update documentation and add project configuration
- Update changelog with complete project history
- Update CLAUDE.md with comprehensive release automation guidelines [skip ci]

### ⚡ Performance

- Optimize CI workflow for faster execution
- Implement aggressive caching to eliminate crates.io downloads

### 🎨 Styling

- Fix formatting in test file
- Apply rustfmt formatting

### ⚙️ Miscellaneous Tasks

- Update workflows to use mise commands
- Remove unnecessary files
- Clean up redundant workflows and update labeler
