#!/bin/sh
# Version Validation Script for LazyCelery
# Validates that all version references are consistent across project files

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() { printf "${GREEN}[INFO]${NC} %s\n" "$1"; }
log_warn() { printf "${YELLOW}[WARN]${NC} %s\n" "$1" >&2; }
log_error() { printf "${RED}[ERROR]${NC} %s\n" "$1" >&2; }

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_ROOT"

ERRORS=0

log_info "Validating version consistency..."

# Get version from Cargo.toml
CARGO_VERSION=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')

if [ -z "$CARGO_VERSION" ]; then
    log_error "Could not determine version from Cargo.toml"
    exit 1
fi

log_info "Cargo.toml version: $CARGO_VERSION"
DOCKER_RUST=$(grep 'FROM rust:' Dockerfile | sed 's/.*rust:\([0-9.]*\).*/\1/')
# Check .mise.toml rust version
MISE_RUST=$(grep '^rust = ' .mise.toml | sed 's/rust = "\(.*\)"/\1/')

if [ -n "$MISE_RUST" ]; then
    log_info ".mise.toml Rust: $MISE_RUST"
fi

# Check Dockerfile rust version
DOCKER_RUST=$(grep 'FROM rust:' Dockerfile | sed 's/.*rust:\([0-9.]*\).*/\1/')

if [ -n "$DOCKER_RUST" ]; then
    log_info "Dockerfile Rust: $DOCKER_RUST"
fi

# Validate rust-version in Cargo.toml matches mise.toml
CARGO_RUST_VERSION=$(grep '^rust-version = ' Cargo.toml | sed 's/rust-version = "\(.*\)"/\1/')

if [ -n "$CARGO_RUST_VERSION" ]; then
    log_info "Cargo.toml rust-version: $CARGO_RUST_VERSION"
    
    if [ -n "$MISE_RUST" ] && [ "$CARGO_RUST_VERSION" != "$MISE_RUST" ]; then
        log_error "Rust version mismatch: Cargo.toml=$CARGO_RUST_VERSION, .mise.toml=$MISE_RUST"
        ERRORS=$((ERRORS + 1))
    fi
    
    if [ -n "$DOCKER_RUST" ] && [ "$CARGO_RUST_VERSION" != "$DOCKER_RUST" ]; then
        log_error "Rust version mismatch: Cargo.toml=$CARGO_RUST_VERSION, Dockerfile=$DOCKER_RUST"
        ERRORS=$((ERRORS + 1))
    fi
fi

# Check for placeholder hashes (warnings only)
if grep -r "PLACEHOLDER_SHA256" packaging/ >/dev/null 2>&1; then
    log_warn "Found PLACEHOLDER_SHA256 in packaging files (expected for development)"
fi

if [ $ERRORS -eq 0 ]; then
    log_info "All version checks passed!"
    exit 0
else
    log_error "Found $ERRORS version inconsistencies"
    exit 1
fi
