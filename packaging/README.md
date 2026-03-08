# Package Management

This directory contains packaging configurations for various package managers.

## Current Approach

LazyCelery uses **cargo-dist** for automated releases and binary distribution. Package configurations are managed through:

- `.github/workflows/dist.yml` - Automated build and release workflow
- `Cargo.toml` - Package metadata (version, description, author, etc.)

## Installation Methods

### Quick Install (Recommended)

**Linux/macOS:**
```bash
curl -sSL https://raw.githubusercontent.com/Fguedes90/lazycelery/main/install.sh | sh
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/Fguedes90/lazycelery/main/install.ps1 | iex
```

### Package Managers

| Package Manager | Command | Platform |
|----------------|---------|----------|
| Homebrew | `brew install lazycelery` | macOS/Linux |
| Scoop | `scoop install lazycelery` | Windows |
| AUR | `yay -S lazycelery` | Arch Linux |
| Cargo | `cargo install lazycelery` | All |

### Manual Download

Pre-built binaries are available on the [GitHub Releases](https://github.com/Fguedes90/lazycelery/releases) page.

## Package Manager Files

This directory contains manual package configurations for reference:
- `homebrew/lazycelery.rb` - Homebrew formula
- `scoop/lazycelery.json` - Scoop manifest

These files are automatically updated during the release process by the CI workflow.
