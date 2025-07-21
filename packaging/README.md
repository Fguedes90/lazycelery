# Package Management

This directory contains packaging configurations for various package managers, all generated from the canonical metadata in `Cargo.toml`.

## Template System

To avoid metadata duplication and ensure consistency across all package managers, we use a template generation system:

### Usage

```bash
# Generate all package files from Cargo.toml metadata
python3 packaging/generate_packages.py
```

This script automatically:
- Extracts metadata from `Cargo.toml` (version, description, author, etc.)
- Generates consistent package configurations for all platforms
- Updates version numbers across all packaging formats

### Supported Package Managers

| Package Manager | File | Platform |
|-----------------|------|----------|
| **AUR Source** | `aur/PKGBUILD` | Arch Linux (build from source) |
| **AUR Binary** | `aur/PKGBUILD-bin` | Arch Linux (pre-built binary) |
| **Homebrew** | `homebrew/lazycelery.rb` | macOS |
| **Chocolatey** | `chocolatey/lazycelery.nuspec` | Windows |
| **Scoop** | `scoop/lazycelery.json` | Windows |
| **Snap** | `snap/snapcraft.yaml` | Ubuntu/Linux |

### Manual Updates Required

After running the generator, you may need to manually update:

1. **SHA256 Hashes**: Update `PLACEHOLDER_SHA256` values after creating release artifacts
2. **Platform-specific details**: Adjust build dependencies or installation steps if needed
3. **Release notes**: Add version-specific changelog entries

### Benefits

- ✅ **Single Source of Truth**: All metadata comes from `Cargo.toml`
- ✅ **Version Consistency**: No more mismatched versions across packages
- ✅ **Reduced Duplication**: Description, license, repository URL only defined once
- ✅ **Easy Updates**: Change once in `Cargo.toml`, regenerate all packages
- ✅ **CI Integration**: Can be automated in release workflows

### Integration with CI/CD

The package generation can be integrated into the automated release workflow:

```yaml
- name: Generate package files
  run: python3 packaging/generate_packages.py

- name: Update SHA256 hashes
  run: |
    # Calculate and update hashes for release artifacts
    # This would be done after building release binaries
```

### Development Workflow

1. **Update version** in `Cargo.toml`
2. **Run generator**: `python3 packaging/generate_packages.py`
3. **Commit changes**: All package files are now updated consistently
4. **Create release**: CI will use the updated package configurations

This approach ensures that package maintainers always have accurate, up-to-date configurations without manual synchronization across multiple files.