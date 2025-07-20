#!/usr/bin/env bash
# Install mise (formerly rtx) - https://mise.jdx.dev/

set -euo pipefail

echo "Installing mise..."

# Install mise
curl https://mise.run | sh

# Add to shell (detect shell)
SHELL_NAME=$(basename "$SHELL")

case "$SHELL_NAME" in
    bash)
        echo 'eval "$(~/.local/bin/mise activate bash)"' >> ~/.bashrc
        echo "Added mise to ~/.bashrc"
        ;;
    zsh)
        echo 'eval "$(~/.local/bin/mise activate zsh)"' >> ~/.zshrc
        echo "Added mise to ~/.zshrc"
        ;;
    fish)
        echo '~/.local/bin/mise activate fish | source' >> ~/.config/fish/config.fish
        echo "Added mise to ~/.config/fish/config.fish"
        ;;
    *)
        echo "Unknown shell: $SHELL_NAME"
        echo "Please manually add mise activation to your shell config"
        echo "See: https://mise.jdx.dev/getting-started.html"
        ;;
esac

echo ""
echo "✅ mise installed successfully!"
echo ""
echo "Please restart your shell or run:"
echo "  source ~/.$SHELL_NAME"rc
echo ""
echo "Then run 'mise --version' to verify installation"