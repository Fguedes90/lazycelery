#!/bin/sh
# LazyCelery Install Script
# Usage: curl -sSL https://raw.githubusercontent.com/Fguedes90/lazycelery/main/install.sh | sh
# Or:   wget -qO- https://raw.githubusercontent.com/Fguedes90/lazycelery/main/install.sh | sh

set -e

REPO="Fguedes90/lazycelery"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
FORCE="${FORCE:-false}"

# Colors (disable if not a tty)
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    NC='\033[0m'
else
    RED=''
    GREEN=''
    YELLOW=''
    NC=''
fi

log_info() { printf "${GREEN}[INFO]${NC} %s\n" "$1"; }
log_warn() { printf "${YELLOW}[WARN]${NC} %s\n" "$1"; }
log_error() { printf "${RED}[ERROR]${NC} %s\n" "$1" >&2; }

# Detect OS and architecture
detect_os() {
    case "$(uname -s)" in
        Linux*)     echo "linux";;
        Darwin*)    echo "macos";;
        CYGWIN*|MINGW*|MSYS*) echo "windows";;
        *)          echo "unknown";;
    esac
}

detect_arch() {
    case "$(uname -m)" in
        x86_64)     echo "x86_64";;
        aarch64|arm64) echo "aarch64";;
        *)          echo "x86_64";;
    esac
}

# Get latest version from GitHub API
get_latest_version() {
    if command -v curl >/dev/null 2>&1; then
        VERSION=$(curl -sSL https://api.github.com/repos/${REPO}/releases/latest | grep '"tag_name"' | cut -d'"' -f4 | cut -c2-)
    elif command -v wget >/dev/null 2>&1; then
        VERSION=$(wget -qO- https://api.github.com/repos/${REPO}/releases/latest | grep '"tag_name"' | cut -d'"' -f4 | cut -c2-)
    else
        log_error "Neither curl nor wget found. Please install one of them."
        exit 1
    fi
    
    if [ -z "$VERSION" ]; then
        log_error "Could not fetch latest version"
        exit 1
    fi
    echo "$VERSION"
}

# Download and install binary
install_binary() {
    OS=$(detect_os)
    ARCH=$(detect_arch)
    VERSION="$1"
    
    log_info "Installing LazyCelery v${VERSION} for ${OS}-${ARCH}"
    
    # Set filename based on OS
    case "$OS" in
        linux)
            FILENAME="lazycelery-linux-${ARCH}.tar.gz"
            ;;
        macos)
            FILENAME="lazycelery-macos-${ARCH}.tar.gz"
            ;;
        windows)
            FILENAME="lazycelery-windows-${ARCH}.zip"
            ;;
        *)
            log_error "Unsupported OS: ${OS}"
            exit 1
            ;;
    esac
    
    URL="https://github.com/${REPO}/releases/download/v${VERSION}/${FILENAME}"
    
    log_info "Downloading from: $URL"
    
    # Create temp directory
    TMPDIR=$(mktemp -d)
    ARCHIVE="${TMPDIR}/lazycelery.${FILENAME##*.}"
    
    # Download
    if command -v curl >/dev/null 2>&1; then
        curl -sSL "$URL" -o "$ARCHIVE"
    else
        wget -q "$URL" -O "$ARCHIVE"
    fi
    
    # Extract
    log_info "Extracting..."
    case "$FILENAME" in
        *.tar.gz)
            tar -xzf "$ARCHIVE" -C "$TMPDIR"
            ;;
        *.zip)
            unzip -q "$ARCHIVE" -d "$TMPDIR"
            ;;
    esac
    
    # Find binary
    BINARY=$(find "$TMPDIR" -name "lazycelery*" -type f -perm -u+x | head -1)
    
    if [ -z "$BINARY" ]; then
        log_error "Could not find binary in archive"
        rm -rf "$TMPDIR"
        exit 1
    fi
    
    # Ensure install directory exists
    mkdir -p "$INSTALL_DIR"
    
    # Check if binary exists
    if [ -f "${INSTALL_DIR}/lazycelery" ] && [ "$FORCE" != "true" ]; then
        log_warn "Binary already exists at ${INSTALL_DIR}/lazycelery"
        log_warn "Set FORCE=true to overwrite"
        rm -rf "$TMPDIR"
        exit 1
    fi
    
    # Install
    cp "$BINARY" "${INSTALL_DIR}/lazycelery"
    chmod +x "${INSTALL_DIR}/lazycelery"
    
    # Cleanup
    rm -rf "$TMPDIR"
    
    log_info "Installed successfully to ${INSTALL_DIR}/lazycelery"
    
    # Verify
    if "${INSTALL_DIR}/lazycelery" --version >/dev/null 2>&1; then
        log_info "Verification passed!"
        "${INSTALL_DIR}/lazycelery" --version
    else
        log_warn "Verification failed, but binary was installed"
    fi
    
    log_info "Add ${INSTALL_DIR} to your PATH if not already present"
}

# Main
main() {
    log_info "LazyCelery Installer"
    log_info "Repository: https://github.com/${REPO}"
    echo ""
    
    VERSION="${VERSION:-$(get_latest_version)}"
    log_info "Target version: $VERSION"
    
    install_binary "$VERSION"
    
    echo ""
    log_info "Installation complete!"
    log_info "Run 'lazycelery --help' to get started"
}

main "$@"
