#!/bin/bash
set -euo pipefail

echo "╔════════════════════════════════════════════════╗"
echo "║        NetPulse — Linux Setup Script           ║"
echo "╚════════════════════════════════════════════════╝"
echo ""

# Check if running as root
if [ "$EUID" -eq 0 ]; then
    echo "⚠️  Please run this script as a normal user (not root)."
    echo "   The script will use sudo when needed."
    exit 1
fi

# Detect package manager
if command -v apt-get &> /dev/null; then
    PKG_MANAGER="apt"
elif command -v dnf &> /dev/null; then
    PKG_MANAGER="dnf"
elif command -v pacman &> /dev/null; then
    PKG_MANAGER="pacman"
else
    echo "❌ Unsupported package manager. Please install libpcap-dev manually."
    exit 1
fi

echo "📦 Installing system dependencies..."
case $PKG_MANAGER in
    apt)
        sudo apt-get update -qq
        sudo apt-get install -y libpcap-dev build-essential pkg-config \
            libssl-dev libgtk-3-dev libwebkit2gtk-4.1-dev \
            libappindicator3-dev librsvg2-dev
        ;;
    dnf)
        sudo dnf install -y libpcap-devel gcc pkg-config \
            openssl-devel gtk3-devel webkit2gtk4.1-devel \
            libappindicator-gtk3-devel librsvg2-devel
        ;;
    pacman)
        sudo pacman -S --noconfirm libpcap base-devel pkg-config \
            openssl gtk3 webkit2gtk-4.1 libappindicator-gtk3 librsvg
        ;;
esac

echo ""
echo "✅ System dependencies installed."
echo ""

# Check for Rust
if ! command -v rustc &> /dev/null; then
    echo "🦀 Rust is not installed. Installing via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "🦀 Rust is already installed: $(rustc --version)"
fi

# Check for Node.js
if ! command -v node &> /dev/null; then
    echo "📦 Node.js is not installed."
    echo "   Please install Node.js 18+ from: https://nodejs.org/"
    exit 1
else
    echo "📦 Node.js: $(node --version)"
fi

echo ""
echo "📥 Installing npm dependencies..."
npm install

echo ""
echo "🔨 Building NetPulse..."
npm run tauri build

echo ""
echo "🔐 Setting network capture capabilities..."
BINARY_PATH="./src-tauri/target/release/netpulse"
if [ -f "$BINARY_PATH" ]; then
    sudo setcap cap_net_raw+ep "$BINARY_PATH"
    echo "   ✅ CAP_NET_RAW granted to $BINARY_PATH"
else
    echo "   ⚠️  Binary not found at $BINARY_PATH"
    echo "   Run: sudo setcap cap_net_raw+ep <path-to-binary>"
fi

echo ""
echo "╔════════════════════════════════════════════════╗"
echo "║              Setup Complete! 🎉               ║"
echo "╠════════════════════════════════════════════════╣"
echo "║                                                ║"
echo "║  To run in dev mode:                           ║"
echo "║    npm run tauri dev                           ║"
echo "║                                                ║"
echo "║  To run the built binary:                      ║"
echo "║    ./src-tauri/target/release/netpulse         ║"
echo "║                                                ║"
echo "║  ⚠️  Remember to download GeoLite2-City.mmdb    ║"
echo "║  and place it in ~/.local/share/netpulse/      ║"
echo "║                                                ║"
echo "╚════════════════════════════════════════════════╝"
