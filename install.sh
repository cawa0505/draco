#!/bin/sh
set -e

echo "Installing Draco..."

# Detect OS
OS="$(uname -s)"
case "$OS" in
    Linux*)  OS_NAME="linux" ;;
    Darwin*) OS_NAME="macos" ;;
    *)       echo "Unsupported OS: $OS"; exit 1 ;;
esac

# Detect Architecture
ARCH="$(uname -m)"
case "$ARCH" in
    x86_64)           ARCH_NAME="x86-64" ;;
    arm64 | aarch64)  ARCH_NAME="arm64" ;;
    *)                echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

TAR_NAME="draco-${OS_NAME}-${ARCH_NAME}.tar.gz"
URL="https://github.com/0xchasercat/draco/releases/latest/download/$TAR_NAME"

INSTALL_DIR="$HOME/.draco"
BIN_DIR="$INSTALL_DIR/bin"
mkdir -p "$BIN_DIR"

echo "Downloading $TAR_NAME..."
TMP_DIR=$(mktemp -d)

# Download and extract
if curl -fsSL "$URL" | tar -xz -C "$TMP_DIR"; then
    # Locate the draco binary (even if it's inside a nested folder in the tarball) and move it
    find "$TMP_DIR" -type f -name "draco" -exec mv {} "$BIN_DIR/draco" \;
    chmod +x "$BIN_DIR/draco"
else
    echo "Failed to download or extract. Ensure a release exists for ${OS_NAME}-${ARCH_NAME}."
    rm -rf "$TMP_DIR"
    exit 1
fi
rm -rf "$TMP_DIR"

# Detect Shell and setup PATH
SHELL_NAME="$(basename "$SHELL")"
case "$SHELL_NAME" in
    zsh)
        PROFILE="$HOME/.zshrc"
        LINE="export PATH=\"\$HOME/.draco/bin:\$PATH\""
        ;;
    bash)
        PROFILE="$HOME/.bashrc"
        LINE="export PATH=\"\$HOME/.draco/bin:\$PATH\""
        ;;
    fish)
        PROFILE="$HOME/.config/fish/config.fish"
        LINE="fish_add_path \"\$HOME/.draco/bin\""
        mkdir -p "$HOME/.config/fish"
        ;;
    *)
        PROFILE="$HOME/.profile"
        LINE="export PATH=\"\$HOME/.draco/bin:\$PATH\""
        ;;
esac

# Inject into PATH if not already present
if ! grep -q "\.draco/bin" "$PROFILE" 2>/dev/null; then
    echo "" >> "$PROFILE"
    echo "# Draco Web Scraper" >> "$PROFILE"
    echo "$LINE" >> "$PROFILE"
    echo "Added Draco to PATH in $PROFILE"
fi

echo ""
echo "✨ Draco installed successfully to $BIN_DIR/draco!"
echo "Please restart your terminal or run:"
echo "  source $PROFILE"
echo ""
echo "Then test it out: draco scrape https://example.com"
