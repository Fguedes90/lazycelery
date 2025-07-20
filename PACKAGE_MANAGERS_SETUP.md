# 📦 Package Managers Setup Guide

This guide explains how to configure automatic distribution to **ALL major package managers** when releases are created.

## 🚀 Package Managers Supported

✅ **Cargo** (Rust) - Already configured  
✅ **Homebrew** (macOS/Linux) - Configured  
✅ **AUR** (Arch Linux) - Configured  
✅ **Scoop** (Windows) - Configured  
✅ **Chocolatey** (Windows) - Configured  
✅ **Snap** (Linux Universal) - Configured  

## 📋 Required Repositories

You need to create these repositories for automated updates:

### 1. Homebrew Tap
```bash
# Create repository: Fguedes90/homebrew-tap
mkdir homebrew-tap && cd homebrew-tap
mkdir Formula
git init
git remote add origin https://github.com/Fguedes90/homebrew-tap.git
echo "# LazyCelery Homebrew Tap" > README.md
git add . && git commit -m "Initial commit"
git push -u origin main
```

### 2. Scoop Bucket
```bash
# Create repository: Fguedes90/scoop-bucket  
mkdir scoop-bucket && cd scoop-bucket
mkdir bucket
git init
git remote add origin https://github.com/Fguedes90/scoop-bucket.git
echo "# LazyCelery Scoop Bucket" > README.md
git add . && git commit -m "Initial commit"
git push -u origin main
```

### 3. AUR Packages
- **Source package**: Register `lazycelery` on AUR
- **Binary package**: Register `lazycelery-bin` on AUR

**To register on AUR:**
1. Create account on https://aur.archlinux.org/
2. Upload SSH public key to AUR
3. Create initial PKGBUILD and submit packages

## 🔑 Required GitHub Secrets

Configure these secrets in GitHub repository settings:

### Essential Secrets

| Secret Name | Description | Required |
|-------------|-------------|----------|
| `CARGO_REGISTRY_TOKEN` | crates.io API token | ✅ Yes |
| `HOMEBREW_TAP_TOKEN` | GitHub token with repo access | ✅ Yes |

### Optional Secrets (skip if not needed)

| Secret Name | Description | Auto-fallback |
|-------------|-------------|---------------|
| `AUR_SSH_KEY` | SSH private key for AUR | Skips AUR if missing |
| `SCOOP_BUCKET_TOKEN` | GitHub token for Scoop bucket | Uses GITHUB_TOKEN |
| `SNAP_STORE_LOGIN` | Snap store login credentials | Builds but doesn't upload |

## 🛠️ Setup Instructions

### 1. Cargo (Already Done)
- ✅ Token configured as `CARGO_REGISTRY_TOKEN`
- ✅ Automatic publishing on release

### 2. Homebrew Setup

```bash
# 1. Create homebrew-tap repository (see above)
# 2. Generate GitHub token with repo permissions
# 3. Add as HOMEBREW_TAP_TOKEN secret

# Test installation after first release:
brew tap Fguedes90/tap
brew install lazycelery
```

### 3. AUR Setup

```bash
# 1. Create AUR account and upload SSH key
# 2. Generate SSH private key for GitHub Actions
ssh-keygen -t rsa -b 4096 -C "github-actions@lazycelery"
# 3. Add private key as AUR_SSH_KEY secret
# 4. Add public key to AUR account

# Test installation after first release:
yay -S lazycelery       # Source package
yay -S lazycelery-bin   # Binary package
```

### 4. Scoop Setup

```bash
# 1. Create scoop-bucket repository (see above)  
# 2. Generate GitHub token (optional, uses GITHUB_TOKEN if not set)
# 3. Add as SCOOP_BUCKET_TOKEN secret

# Test installation after first release:
scoop bucket add lazycelery https://github.com/Fguedes90/scoop-bucket.git
scoop install lazycelery
```

### 5. Chocolatey Setup

```bash
# Manual process (automated update not implemented yet):
# 1. Create account on https://chocolatey.org/
# 2. Generate API key
# 3. Upload package manually or use choco push

# Test installation:
choco install lazycelery
```

### 6. Snap Setup

```bash
# 1. Create account on https://snapcraft.io/
# 2. Register 'lazycelery' name
# 3. Generate login token:
snapcraft export-login --snaps=lazycelery --channels=stable -
# 4. Encode token: base64 snapcraft-login.txt
# 5. Add as SNAP_STORE_LOGIN secret

# Test installation after first release:
snap install lazycelery
```

## 🔄 Automated Release Process

When a release is created (via git tag), the workflow automatically:

1. **Builds** cross-platform binaries
2. **Publishes** to crates.io
3. **Updates** Homebrew formula with new version and checksum
4. **Updates** AUR packages (source and binary) with new checksums
5. **Updates** Scoop manifest with new version and checksum
6. **Builds** and uploads Snap package (if credentials configured)
7. **Creates** GitHub release with all binaries

## 📊 Installation Methods Summary

After setup, users can install LazyCelery using:

```bash
# Rust/Cargo (All platforms)
cargo install lazycelery

# macOS/Linux - Homebrew
brew tap Fguedes90/tap && brew install lazycelery

# Arch Linux - AUR
yay -S lazycelery          # Source
yay -S lazycelery-bin      # Binary

# Windows - Scoop
scoop bucket add lazycelery https://github.com/Fguedes90/scoop-bucket.git
scoop install lazycelery

# Windows - Chocolatey (when available)
choco install lazycelery

# Linux - Snap
snap install lazycelery

# Manual download
# Download from GitHub releases
```

## 🚨 Troubleshooting

### Common Issues

**Homebrew formula fails:**
- Check HOMEBREW_TAP_TOKEN has repo permissions
- Verify homebrew-tap repository exists and is public

**AUR update fails:**
- Check AUR_SSH_KEY is valid private key
- Verify SSH public key is added to AUR account
- Ensure packages are registered on AUR first

**Scoop update fails:**
- Check scoop-bucket repository exists
- Verify token has write permissions

**Snap build/upload fails:**
- Check SNAP_STORE_LOGIN is valid base64-encoded token
- Verify 'lazycelery' name is registered on Snap Store

### Manual Verification

```bash
# Verify all packages install correctly:
docker run --rm -it ubuntu:latest bash -c "
  apt update && apt install -y snapd &&
  snap install lazycelery &&
  lazycelery --help
"

# Test Homebrew formula:
docker run --rm -it homebrew/brew bash -c "
  brew tap Fguedes90/tap &&
  brew install lazycelery &&
  lazycelery --help
"
```

## 📈 Package Manager Analytics

Monitor package downloads:
- **Cargo**: https://crates.io/crates/lazycelery/stats
- **Homebrew**: View in tap repository insights
- **AUR**: Check package pages for vote/popularity
- **Scoop**: Repository insights
- **Snap**: https://snapcraft.io/lazycelery (when live)

## 🔮 Future Package Managers

Potential additions:
- **WinGet** (Windows Package Manager)
- **Nix** (NixOS/Nix package manager)  
- **pkgsrc** (NetBSD/cross-platform)
- **Alpine APK** (Alpine Linux)
- **Debian APT** (Debian/Ubuntu PPA)
- **RPM** (RedHat/Fedora/SUSE)

Each would follow similar pattern of automated updates on release.