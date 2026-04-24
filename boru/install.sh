#!/usr/bin/env bash
# ╔═══════════════════════════════════════════════════════════╗
# ║  BORU Universal Installer 🥊                             ║
# ║  "What runs here, stays here."                           ║
# ║                                                          ║
# ║  Supports: Debian/Ubuntu, Fedora/RHEL, Arch, macOS       ║
# ║  Usage: curl -sSf <url> | bash                          ║
# ║  Or:    ./install.sh                                     ║
# ╚═══════════════════════════════════════════════════════════╝

set -euo pipefail

BORU_VERSION="0.3.0"
BORU_REPO="https://github.com/sayan5069/Momo.co"
INSTALL_PREFIX="${BORU_INSTALL_PREFIX:-/usr/local}"

# ── Color helpers ──────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

info()    { echo -e "${BLUE}[INFO]${NC} $*"; }
success() { echo -e "${GREEN}[OK]${NC} $*"; }
warn()    { echo -e "${YELLOW}[WARN]${NC} $*"; }
error()   { echo -e "${RED}[ERROR]${NC} $*"; exit 1; }

# ── Platform detection ─────────────────────────────────────
detect_os() {
    local os
    os="$(uname -s)"
    case "$os" in
        Linux)  echo "linux" ;;
        Darwin) echo "macos" ;;
        *)      error "Unsupported OS: $os. BORU runs on Linux and macOS only." ;;
    esac
}

detect_arch() {
    local arch
    arch="$(uname -m)"
    case "$arch" in
        x86_64|amd64)  echo "x86_64" ;;
        aarch64|arm64) echo "aarch64" ;;
        *)             error "Unsupported architecture: $arch. BORU supports x86_64 and aarch64." ;;
    esac
}

detect_distro() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        case "$ID" in
            ubuntu|debian|linuxmint|pop|elementary|zorin|kali|raspbian)
                echo "debian"
                ;;
            fedora|rhel|centos|rocky|alma|ol|nobara)
                echo "rpm"
                ;;
            arch|manjaro|endeavouros|garuda)
                echo "arch"
                ;;
            opensuse*|sles)
                echo "suse"
                ;;
            void)
                echo "void"
                ;;
            alpine)
                echo "alpine"
                ;;
            *)
                # Try to detect based on package manager
                if command -v apt-get &>/dev/null; then
                    echo "debian"
                elif command -v dnf &>/dev/null || command -v yum &>/dev/null; then
                    echo "rpm"
                elif command -v pacman &>/dev/null; then
                    echo "arch"
                elif command -v zypper &>/dev/null; then
                    echo "suse"
                else
                    echo "unknown"
                fi
                ;;
        esac
    elif [ "$(detect_os)" = "macos" ]; then
        echo "macos"
    else
        echo "unknown"
    fi
}

# ── Dependency checks ─────────────────────────────────────
check_rust() {
    if ! command -v rustc &>/dev/null; then
        warn "Rust is not installed."
        info "Installing Rust via rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        # shellcheck source=/dev/null
        source "$HOME/.cargo/env"
        success "Rust installed: $(rustc --version)"
    else
        local version
        version=$(rustc --version | awk '{print $2}')
        info "Rust found: v$version"

        # Check minimum version (1.75+)
        local major minor
        major=$(echo "$version" | cut -d. -f1)
        minor=$(echo "$version" | cut -d. -f2)
        if [ "$major" -lt 1 ] || ([ "$major" -eq 1 ] && [ "$minor" -lt 75 ]); then
            warn "Rust $version is too old. Minimum: 1.75"
            info "Updating Rust..."
            rustup update stable
        fi
    fi
}

check_build_deps() {
    local os="$1"
    local distro="$2"

    info "Checking build dependencies..."

    if [ "$os" = "linux" ]; then
        case "$distro" in
            debian)
                if ! dpkg -l | grep -q build-essential 2>/dev/null; then
                    info "Installing build-essential..."
                    sudo apt-get update -qq
                    sudo apt-get install -y build-essential pkg-config
                fi
                ;;
            rpm)
                if ! rpm -q gcc &>/dev/null 2>/dev/null; then
                    info "Installing build tools..."
                    if command -v dnf &>/dev/null; then
                        sudo dnf install -y gcc gcc-c++ make pkg-config
                    else
                        sudo yum install -y gcc gcc-c++ make pkg-config
                    fi
                fi
                ;;
            arch)
                if ! pacman -Qi base-devel &>/dev/null 2>/dev/null; then
                    info "Installing base-devel..."
                    sudo pacman -S --noconfirm --needed base-devel
                fi
                ;;
            suse)
                if ! rpm -q gcc &>/dev/null 2>/dev/null; then
                    info "Installing build pattern..."
                    sudo zypper install -y -t pattern devel_basis
                fi
                ;;
        esac
    elif [ "$os" = "macos" ]; then
        if ! xcode-select -p &>/dev/null; then
            info "Installing Xcode Command Line Tools..."
            xcode-select --install
            warn "Please complete the Xcode installation and re-run this script."
            exit 0
        fi
    fi

    success "Build dependencies satisfied."
}

# ── Build from source ──────────────────────────────────────
build_boru() {
    local src_dir="$1"

    info "Building BORU v${BORU_VERSION} (release mode)..."
    echo ""

    cd "$src_dir/boru"
    cargo build --release 2>&1 | tail -5

    # Verify binary size (Gate 1: Zero Bloat Law)
    local binary_path="target/release/boru"
    if [ ! -f "$binary_path" ]; then
        error "Build failed: binary not found at $binary_path"
    fi

    local size_bytes
    size_bytes=$(stat -c%s "$binary_path" 2>/dev/null || stat -f%z "$binary_path" 2>/dev/null)
    local size_mb=$((size_bytes / 1048576))

    if [ "$size_mb" -ge 10 ]; then
        warn "Binary size ${size_mb}MB exceeds 10MB budget! (Gate 1 violation)"
    else
        success "Binary size: ${size_mb}MB (within 10MB budget ✓)"
    fi

    echo "$binary_path"
}

# ── Install binary ─────────────────────────────────────────
install_binary() {
    local binary_path="$1"
    local install_dir="${INSTALL_PREFIX}/bin"

    info "Installing BORU to ${install_dir}/boru..."

    if [ -w "$install_dir" ]; then
        cp "$binary_path" "${install_dir}/boru"
        chmod 755 "${install_dir}/boru"
    else
        sudo install -Dm755 "$binary_path" "${install_dir}/boru"
    fi

    success "Binary installed: ${install_dir}/boru"
}

# ── Setup runtime directories ─────────────────────────────
setup_runtime() {
    info "Setting up runtime directories..."

    # Socket directory
    mkdir -p /tmp/momo
    chmod 0755 /tmp/momo

    # Quarantine directory
    mkdir -p /tmp/momo/quarantine
    chmod 0700 /tmp/momo/quarantine

    # Audit log directory
    if [ -w /var/log ] || [ "$(id -u)" -eq 0 ]; then
        sudo mkdir -p /var/log/boru
        sudo chmod 0755 /var/log/boru
    else
        mkdir -p "$HOME/.local/share/boru/logs"
        info "Audit logs: $HOME/.local/share/boru/logs (non-root install)"
    fi

    # Hash database directory
    if [ -w /var/lib ] || [ "$(id -u)" -eq 0 ]; then
        sudo mkdir -p /var/lib/boru
        sudo chmod 0755 /var/lib/boru
    else
        mkdir -p "$HOME/.local/share/boru/db"
        info "Hash DB: $HOME/.local/share/boru/db (non-root install)"
    fi

    success "Runtime directories created."
}

# ── Main ───────────────────────────────────────────────────
main() {
    echo ""
    echo -e "${BOLD}${CYAN}"
    echo "  ╔═══════════════════════════════════════════╗"
    echo "  ║         BORU Installer 🥊  v${BORU_VERSION}         ║"
    echo "  ║    \"What runs here, stays here.\"          ║"
    echo "  ╚═══════════════════════════════════════════╝"
    echo -e "${NC}"
    echo ""

    local os distro arch
    os=$(detect_os)
    arch=$(detect_arch)
    distro=$(detect_distro)

    info "Detected: ${os} / ${distro} / ${arch}"
    echo ""

    # Check and install Rust
    check_rust

    # Check build dependencies
    check_build_deps "$os" "$distro"

    # Determine source directory
    local src_dir
    if [ -f "boru/Cargo.toml" ]; then
        # Running from repo root
        src_dir="$(pwd)"
    elif [ -f "Cargo.toml" ] && grep -q 'name = "boru"' Cargo.toml; then
        # Running from boru/ directory
        src_dir="$(pwd)/.."
    else
        # Clone the repo
        info "Cloning BORU repository..."
        local tmp_dir
        tmp_dir="$(mktemp -d)"
        git clone --depth 1 "$BORU_REPO" "$tmp_dir"
        src_dir="$tmp_dir"
    fi

    # Build
    local binary_path
    binary_path=$(build_boru "$src_dir")
    echo ""

    # Install
    install_binary "$src_dir/boru/$binary_path"

    # Setup runtime
    setup_runtime
    echo ""

    # Verify
    info "Verifying installation..."
    if command -v boru &>/dev/null; then
        local installed_version
        installed_version=$(boru --version 2>/dev/null || echo "unknown")
        success "boru is available: $installed_version"
    else
        warn "boru not found in PATH. You may need to add ${INSTALL_PREFIX}/bin to your PATH:"
        echo "  export PATH=\"${INSTALL_PREFIX}/bin:\$PATH\""
    fi

    echo ""
    echo -e "${BOLD}${GREEN}"
    echo "  ╔═══════════════════════════════════════════════════════╗"
    echo "  ║  Installation complete! 🥊                           ║"
    echo "  ╠═══════════════════════════════════════════════════════╣"
    echo "  ║                                                       ║"
    echo "  ║  Quick Start:                                         ║"
    echo "  ║    boru daemon &     — Start socket daemon            ║"
    echo "  ║    boru tui          — Launch TUI dashboard           ║"
    echo "  ║    boru cage --input <file> --mode strict             ║"
    echo "  ║    boru scan --path <dir>                             ║"
    echo "  ║    boru --help       — See all commands               ║"
    echo "  ║                                                       ║"
    echo "  ║  Socket: /tmp/momo/boru.sock                          ║"
    echo "  ╚═══════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

main "$@"
