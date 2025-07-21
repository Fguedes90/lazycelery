#!/usr/bin/env python3
"""
Package Metadata Generator for LazyCelery

This script generates package configuration files from Cargo.toml metadata,
reducing duplication and ensuring consistency across package managers.

Usage:
    python3 packaging/generate_packages.py [--calculate-hashes]
"""

import os
import sys
import json
import re
import hashlib
import urllib.request
import tempfile
from pathlib import Path
from typing import Dict, Any, Optional


def parse_toml_simple(content: str) -> Dict[str, Any]:
    """Simple TOML parser for Cargo.toml package section"""
    lines = content.split('\n')
    in_package = False
    package_data = {}
    
    for line in lines:
        line = line.strip()
        if line == '[package]':
            in_package = True
            continue
        elif line.startswith('[') and line != '[package]':
            in_package = False
            continue
            
        if in_package and '=' in line:
            key, value = line.split('=', 1)
            key = key.strip()
            value = value.strip().strip('"\'')
            
            # Handle arrays
            if value.startswith('[') and value.endswith(']'):
                value = value[1:-1]
                if value:
                    items = [item.strip().strip('"\'') for item in value.split(',')]
                    package_data[key] = items
                else:
                    package_data[key] = []
            else:
                package_data[key] = value
    
    return package_data


def load_cargo_metadata(cargo_toml_path: str) -> Dict[str, Any]:
    """Load metadata from Cargo.toml"""
    with open(cargo_toml_path, 'r') as f:
        content = f.read()
    
    package = parse_toml_simple(content)
    
    # Extract author from array format if needed
    authors = package.get('authors', ['Unknown'])
    if isinstance(authors, list):
        authors = authors[0] if authors else 'Unknown'
    
    return {
        'name': package.get('name', 'lazycelery'),
        'version': package.get('version', '0.4.0'),
        'description': package.get('description', 'Terminal UI for monitoring Celery'),
        'authors': [authors],
        'license': package.get('license', 'MIT'),
        'repository': package.get('repository', 'https://github.com/Fguedes90/lazycelery'),
        'homepage': package.get('homepage', 'https://github.com/Fguedes90/lazycelery'),
        'keywords': package.get('keywords', ['celery', 'tui', 'terminal', 'monitoring', 'redis']),
        'categories': package.get('categories', ['command-line-utilities'])
    }


def calculate_sha256_from_url(url: str) -> Optional[str]:
    """
    Calculate SHA256 hash of a file from URL.
    Returns None if the file doesn't exist or there's an error.
    """
    try:
        with urllib.request.urlopen(url) as response:
            hasher = hashlib.sha256()
            while True:
                chunk = response.read(8192)
                if not chunk:
                    break
                hasher.update(chunk)
            return hasher.hexdigest()
    except Exception as e:
        print(f"Warning: Could not calculate hash for {url}: {e}")
        return None


def sanitize_input(value: str, max_length: int = 256, allowed_chars: str = None) -> str:
    """
    Sanitize input to prevent injection attacks.
    
    Args:
        value: Input string to sanitize
        max_length: Maximum allowed length
        allowed_chars: String of allowed characters (if None, uses safe defaults)
    
    Returns:
        Sanitized string
        
    Raises:
        ValueError: If input is invalid
    """
    if not isinstance(value, str):
        raise ValueError("Input must be a string")
    
    if len(value) > max_length:
        raise ValueError(f"Input exceeds maximum length of {max_length}")
    
    if allowed_chars is None:
        # Safe defaults for package names, versions, URLs
        allowed_chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789.-_/:"
    
    for char in value:
        if char not in allowed_chars:
            raise ValueError(f"Invalid character '{char}' in input")
    
    return value


def validate_version(version: str) -> str:
    """Validate semantic version format"""
    # Allow only semantic version format: x.y.z with optional pre-release
    if not re.match(r'^\d+\.\d+\.\d+(-[a-zA-Z0-9\-\.]+)?$', version):
        raise ValueError(f"Invalid version format: {version}")
    return version


def validate_url(url: str) -> str:
    """Validate URL format"""
    if not re.match(r'^https?://[a-zA-Z0-9\-\.]+(/.*)?$', url):
        raise ValueError(f"Invalid URL format: {url}")
    return url


def generate_pkgbuild_source(metadata: Dict[str, Any], calculate_hashes: bool = False) -> str:
    """Generate AUR PKGBUILD for source build"""
    # Validate and sanitize inputs
    name = sanitize_input(metadata['name'], 50, "abcdefghijklmnopqrstuvwxyz0123456789-")
    version = validate_version(metadata['version'])
    repository = validate_url(metadata['repository'])
    homepage = validate_url(metadata['homepage'])
    
    # Calculate SHA256 if requested and possible
    sha256_hash = "PLACEHOLDER_SHA256"
    if calculate_hashes:
        source_url = f"{repository}/archive/v{version}.tar.gz"
        calculated_hash = calculate_sha256_from_url(source_url)
        if calculated_hash:
            sha256_hash = calculated_hash
    
    return f"""# Maintainer: {metadata['authors'][0]}
pkgname={name}
pkgver={version}
pkgrel=1
pkgdesc="{metadata['description']}"
arch=('x86_64')
url="{homepage}"
license=('{metadata['license']}')
depends=('gcc-libs')
makedepends=('rust' 'cargo')
source=("$pkgname-$pkgver.tar.gz::{repository}/archive/v$pkgver.tar.gz")
sha256sums=('{sha256_hash}')

build() {{
    cd "$pkgname-$pkgver"
    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=target
    cargo build --release --locked
}}

check() {{
    cd "$pkgname-$pkgver"
    export RUSTUP_TOOLCHAIN=stable
    cargo test --release --locked
}}

package() {{
    cd "$pkgname-$pkgver"
    install -Dm755 target/release/$pkgname "$pkgdir/usr/bin/$pkgname"
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
    install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
}}"""


def generate_pkgbuild_bin(metadata: Dict[str, Any], calculate_hashes: bool = False) -> str:
    """Generate AUR PKGBUILD for binary release"""
    # Validate and sanitize inputs
    name = sanitize_input(metadata['name'], 50, "abcdefghijklmnopqrstuvwxyz0123456789-")
    version = validate_version(metadata['version'])
    repository = validate_url(metadata['repository'])
    homepage = validate_url(metadata['homepage'])
    
    # Calculate SHA256 if requested and possible
    sha256_hash = "PLACEHOLDER_SHA256"
    if calculate_hashes:
        binary_url = f"{repository}/releases/download/v{version}/{name}-linux-x86_64.tar.gz"
        calculated_hash = calculate_sha256_from_url(binary_url)
        if calculated_hash:
            sha256_hash = calculated_hash
    
    return f"""# Maintainer: {metadata['authors'][0]}
pkgname={name}-bin
pkgver={version}
pkgrel=1
pkgdesc="{metadata['description']} (binary release)"
arch=('x86_64')
url="{homepage}"
license=('{metadata['license']}')
depends=('gcc-libs')
provides=('{name}')
conflicts=('{name}')
source_x86_64=("{repository}/releases/download/v$pkgver/{name}-linux-x86_64.tar.gz")
sha256sums_x86_64=('{sha256_hash}')

package() {{
    install -Dm755 {metadata['name']} "$pkgdir/usr/bin/{metadata['name']}"
    
    # Download and install license and docs
    curl -sL "{metadata['repository']}/raw/v$pkgver/LICENSE" -o LICENSE
    curl -sL "{metadata['repository']}/raw/v$pkgver/README.md" -o README.md
    
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
    install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
}}"""


def generate_homebrew_formula(metadata: Dict[str, Any], calculate_hashes: bool = False) -> str:
    """Generate Homebrew formula"""
    # Validate and sanitize inputs
    name = sanitize_input(metadata['name'], 50, "abcdefghijklmnopqrstuvwxyz0123456789-")
    version = validate_version(metadata['version'])
    repository = validate_url(metadata['repository'])
    homepage = validate_url(metadata['homepage'])
    
    # Calculate SHA256 if requested and possible
    sha256_hash = "PLACEHOLDER_SHA256"
    if calculate_hashes:
        source_url = f"{repository}/archive/v{version}.tar.gz"
        calculated_hash = calculate_sha256_from_url(source_url)
        if calculated_hash:
            sha256_hash = calculated_hash
    
    class_name = name.capitalize()
    return f"""class {class_name} < Formula
  desc "{metadata['description']}"
  homepage "{homepage}"
  url "{repository}/archive/v{version}.tar.gz"
  sha256 "{sha256_hash}"
  license "{metadata['license']}"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "{metadata['name']}", shell_output("#{{bin}}/{metadata['name']} --help")
  end
end"""


def generate_chocolatey_nuspec(metadata: Dict[str, Any], calculate_hashes: bool = False) -> str:
    """Generate Chocolatey package specification"""
    # Validate and sanitize inputs
    name = sanitize_input(metadata['name'], 50, "abcdefghijklmnopqrstuvwxyz0123456789-")
    version = validate_version(metadata['version'])
    repository = validate_url(metadata['repository'])
    
    author = metadata['authors'][0].split('<')[0].strip()
    tags = ' '.join(metadata['keywords'])
    
    return f"""<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://schemas.microsoft.com/packaging/2015/06/nuspec.xsd">
  <metadata>
    <id>{name}</id>
    <version>{version}</version>
    <packageSourceUrl>{repository}/tree/main/packaging/chocolatey</packageSourceUrl>
    <owners>{author}</owners>
    <title>{name.capitalize()}</title>
    <authors>{author}</authors>
    <projectUrl>{repository}</projectUrl>
    <iconUrl>{repository}/raw/main/screenshots/workers-view.png</iconUrl>
    <copyright>2024 {author}</copyright>
    <licenseUrl>{repository}/blob/main/LICENSE</licenseUrl>
    <requireLicenseAcceptance>false</requireLicenseAcceptance>
    <projectSourceUrl>{repository}</projectSourceUrl>
    <docsUrl>{repository}/blob/main/README.md</docsUrl>
    <bugTrackerUrl>{repository}/issues</bugTrackerUrl>
    <tags>{tags}</tags>
    <summary>{metadata['description']}</summary>
    <description>
{metadata['description']}

## Features
- Real-time monitoring of Celery workers and tasks
- Task management (retry, revoke, purge queues)
- Redis broker support with real Celery protocol integration
- Intuitive terminal interface with vim-style navigation
- Cross-platform support (Linux, macOS, Windows)

## Usage
Run `{name}` in your terminal to start monitoring your Celery infrastructure.
    </description>
    <releaseNotes>{repository}/releases</releaseNotes>
  </metadata>
  <files>
    <file src="tools\\**" target="tools" />
  </files>
</package>"""


def generate_scoop_manifest(metadata: Dict[str, Any], calculate_hashes: bool = False) -> str:
    """Generate Scoop package manifest"""
    # Validate and sanitize inputs
    name = sanitize_input(metadata['name'], 50, "abcdefghijklmnopqrstuvwxyz0123456789-")
    version = validate_version(metadata['version'])
    repository = validate_url(metadata['repository'])
    homepage = validate_url(metadata['homepage'])
    
    # Calculate SHA256 if requested and possible
    sha256_hash = "PLACEHOLDER_SHA256"
    if calculate_hashes:
        windows_url = f"{repository}/releases/download/v{version}/{name}-windows-x86_64.zip"
        calculated_hash = calculate_sha256_from_url(windows_url)
        if calculated_hash:
            sha256_hash = calculated_hash
    
    manifest = {
        "version": version,
        "description": metadata['description'],
        "homepage": homepage,
        "license": metadata['license'],
        "architecture": {
            "64bit": {
                "url": f"{repository}/releases/download/v{version}/{name}-windows-x86_64.zip",
                "hash": sha256_hash,
                "extract_dir": ""
            }
        },
        "bin": f"{name}.exe",
        "checkver": {
            "github": repository
        },
        "autoupdate": {
            "architecture": {
                "64bit": {
                    "url": f"{repository}/releases/download/v$version/{name}-windows-x86_64.zip"
                }
            }
        }
    }
    return json.dumps(manifest, indent=4)


def generate_snapcraft_yaml(metadata: Dict[str, Any], calculate_hashes: bool = False) -> str:
    """Generate Snapcraft YAML"""
    # Validate and sanitize inputs
    name = sanitize_input(metadata['name'], 50, "abcdefghijklmnopqrstuvwxyz0123456789-")
    version = validate_version(metadata['version'])
    repository = validate_url(metadata['repository'])
    
    return f"""name: {name}
base: core22
version: '{version}'
summary: {metadata['description']}
description: |
  {metadata['description']}
  
  Features:
  - Real-time monitoring of Celery workers and tasks
  - Task management (retry, revoke, purge queues)
  - Redis broker support with real Celery protocol integration
  - Intuitive terminal interface with vim-style navigation
  - Cross-platform support

grade: stable
confinement: strict

architectures:
  - build-on: amd64
  - build-on: arm64

apps:
  {name}:
    command: bin/{name}
    plugs:
      - network
      - network-bind
      - home

parts:
  {name}:
    plugin: rust
    source: {repository}.git
    source-tag: v{version}
    rust-features: []
    build-packages:
      - build-essential
      - pkg-config
    override-build: |
      craftctl default
      # Strip the binary to reduce size
      strip $CRAFT_PART_INSTALL/bin/{name}"""


def main():
    """Main function to generate all package files"""
    # Parse command line arguments
    calculate_hashes = "--calculate-hashes" in sys.argv
    
    # Get project root directory
    script_dir = Path(__file__).parent.absolute()
    project_root = script_dir.parent
    cargo_toml = project_root / "Cargo.toml"
    
    if not cargo_toml.exists():
        print(f"Error: Cargo.toml not found at {cargo_toml}", file=sys.stderr)
        sys.exit(1)
    
    # Load and validate metadata
    try:
        metadata = load_cargo_metadata(str(cargo_toml))
        # Validate critical metadata fields
        validate_version(metadata['version'])
        validate_url(metadata['repository'])
        validate_url(metadata['homepage'])
        sanitize_input(metadata['name'], 50, "abcdefghijklmnopqrstuvwxyz0123456789-")
        print(f"Generating packages for {metadata['name']} v{metadata['version']}")
    except (ValueError, KeyError) as e:
        print(f"Error: Invalid metadata in Cargo.toml: {e}", file=sys.stderr)
        sys.exit(1)
    
    if calculate_hashes:
        print("\nCalculating SHA256 hashes from release artifacts...")
    
    # Generate all package files
    generators = [
        ("aur/PKGBUILD", generate_pkgbuild_source),
        ("aur/PKGBUILD-bin", generate_pkgbuild_bin),
        ("homebrew/lazycelery.rb", generate_homebrew_formula),
        ("chocolatey/lazycelery.nuspec", generate_chocolatey_nuspec),
        ("scoop/lazycelery.json", generate_scoop_manifest),
        ("snap/snapcraft.yaml", generate_snapcraft_yaml),
    ]
    
    for file_path, generator in generators:
        full_path = script_dir / file_path
        try:
            content = generator(metadata, calculate_hashes)
            
            # Ensure directory exists
            full_path.parent.mkdir(parents=True, exist_ok=True)
            
            # Write file
            with open(full_path, 'w') as f:
                f.write(content)
            
            print(f"Generated: {file_path}")
        except Exception as e:
            print(f"Error generating {file_path}: {e}", file=sys.stderr)
            continue
    
    print(f"\nAll package files generated successfully!")
    if not calculate_hashes:
        print(f"Note: Use --calculate-hashes to compute real SHA256 hashes from release artifacts.")
    else:
        print(f"Note: SHA256 hashes calculated from available release artifacts.")


if __name__ == "__main__":
    main()