#!/usr/bin/env bash
# ╔═══════════════════════════════════════════════════════════╗
# ║  SANDBOX Package Builder                                     ║
# ║  Builds .deb, .rpm, and Homebrew-ready tarballs          ║
# ╚═══════════════════════════════════════════════════════════╝

set -euo pipefail

SANDBOX_VERSION="0.3.0"
SANDBOX_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BUILD_DIR="${SANDBOX_DIR}/build-pkg"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

info()    { echo -e "${BLUE}[INFO]${NC} $*"; }
success() { echo -e "${GREEN}[OK]${NC} $*"; }
warn()    { echo -e "${YELLOW}[WARN]${NC} $*"; }
error()   { echo -e "${RED}[ERROR]${NC} $*"; exit 1; }

# ── Build release binary ──────────────────────────────────
build_release() {
    info "Building SANDBOX v${SANDBOX_VERSION} (release mode)..."
    cd "$SANDBOX_DIR"
    cargo build --release

    local binary="target/release/sandbox"
    if [ ! -f "$binary" ]; then
        error "Build failed: $binary not found"
    fi

    local size_bytes
    size_bytes=$(stat -c%s "$binary" 2>/dev/null || stat -f%z "$binary" 2>/dev/null)
    local size_mb=$((size_bytes / 1048576))

    if [ "$size_mb" -ge 10 ]; then
        error "Binary ${size_mb}MB exceeds 10MB budget (Gate 1 violation)!"
    fi

    success "Build complete: ${size_mb}MB"
}

# ── Build .deb package ─────────────────────────────────────
build_deb() {
    if ! command -v dpkg-deb &>/dev/null; then
        warn "dpkg-deb not found — skipping .deb build"
        return
    fi

    info "Building .deb package..."

    local deb_root="${BUILD_DIR}/deb"
    rm -rf "$deb_root"

    # Create directory structure
    mkdir -p "${deb_root}/DEBIAN"
    mkdir -p "${deb_root}/usr/bin"
    mkdir -p "${deb_root}/usr/share/doc/sandbox"

    # Control file
    cat > "${deb_root}/DEBIAN/control" << EOF
Package: sandbox
Version: ${SANDBOX_VERSION}
Section: utils
Priority: optional
Architecture: $(dpkg --print-architecture 2>/dev/null || echo "amd64")
Maintainer: SANDBOX Team <sandbox@projectruthenlabs.dev>
Description: Security Cage engine for AI-generated code — Project RUTHENLABS
 SANDBOX intercepts and sandboxes AI-generated code before it touches your
 system using WebAssembly (wasmtime) sandboxing. Zero network calls.
 Tamper-proof audit logging. TUI dashboard. Quarantine & rollback.
 .
 "What runs here, stays here."
Homepage: https://github.com/sayan5069/RuthenLabs.co
EOF

    # postinst
    cp "${SANDBOX_DIR}/packaging/debian/sandbox.postinst" "${deb_root}/DEBIAN/postinst"
    chmod 755 "${deb_root}/DEBIAN/postinst"

    # postrm
    cp "${SANDBOX_DIR}/packaging/debian/sandbox.postrm" "${deb_root}/DEBIAN/postrm"
    chmod 755 "${deb_root}/DEBIAN/postrm"

    # Binary
    cp "${SANDBOX_DIR}/target/release/sandbox" "${deb_root}/usr/bin/sandbox"
    chmod 755 "${deb_root}/usr/bin/sandbox"

    # Docs
    cp "${SANDBOX_DIR}/README.md" "${deb_root}/usr/share/doc/sandbox/"
    cp "${SANDBOX_DIR}/ARCHITECTURE.md" "${deb_root}/usr/share/doc/sandbox/"
    cp "${SANDBOX_DIR}/AGENTS.md" "${deb_root}/usr/share/doc/sandbox/"
    cp "${SANDBOX_DIR}/CHANGELOG.md" "${deb_root}/usr/share/doc/sandbox/"

    # Build
    local deb_file="${BUILD_DIR}/sandbox_${SANDBOX_VERSION}_$(dpkg --print-architecture 2>/dev/null || echo "amd64").deb"
    dpkg-deb --build "$deb_root" "$deb_file"

    success "DEB package: $deb_file"
}

# ── Build .rpm package ─────────────────────────────────────
build_rpm() {
    if ! command -v rpmbuild &>/dev/null; then
        warn "rpmbuild not found — skipping .rpm build"
        warn "Install with: sudo dnf install rpm-build (Fedora) or sudo apt install rpm (Debian)"
        return
    fi

    info "Building .rpm package..."

    local rpm_root="${BUILD_DIR}/rpm"
    rm -rf "$rpm_root"
    mkdir -p "${rpm_root}"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

    # Create tarball
    local tarball_name="sandbox-${SANDBOX_VERSION}"
    local tarball="${rpm_root}/SOURCES/${tarball_name}.tar.gz"

    cd "$SANDBOX_DIR/.."
    tar czf "$tarball" \
        --transform "s,^sandbox,${tarball_name}," \
        --exclude='target' \
        --exclude='.git' \
        --exclude='build-pkg' \
        sandbox/

    # Copy spec
    cp "${SANDBOX_DIR}/packaging/rpm/sandbox.spec" "${rpm_root}/SPECS/"

    # Build
    rpmbuild \
        --define "_topdir ${rpm_root}" \
        -bb "${rpm_root}/SPECS/sandbox.spec"

    local rpm_file
    rpm_file=$(find "${rpm_root}/RPMS" -name "*.rpm" -print -quit)
    if [ -n "$rpm_file" ]; then
        cp "$rpm_file" "${BUILD_DIR}/"
        success "RPM package: ${BUILD_DIR}/$(basename "$rpm_file")"
    fi
}

# ── Build tarball (for Homebrew / manual install) ──────────
build_tarball() {
    info "Building source tarball..."

    local tarball="${BUILD_DIR}/sandbox-${SANDBOX_VERSION}.tar.gz"

    cd "$SANDBOX_DIR/.."
    tar czf "$tarball" \
        --transform "s,^sandbox,sandbox-${SANDBOX_VERSION}," \
        --exclude='target' \
        --exclude='.git' \
        --exclude='build-pkg' \
        sandbox/

    local sha256
    sha256=$(sha256sum "$tarball" 2>/dev/null | awk '{print $1}' || shasum -a 256 "$tarball" | awk '{print $1}')
    success "Tarball: $tarball"
    info "SHA-256: $sha256"

    # Update Homebrew formula with real hash
    if [ -f "${SANDBOX_DIR}/packaging/macos/sandbox.rb" ]; then
        sed -i.bak "s/PLACEHOLDER_SHA256/${sha256}/" "${SANDBOX_DIR}/packaging/macos/sandbox.rb"
        rm -f "${SANDBOX_DIR}/packaging/macos/sandbox.rb.bak"
        info "Updated Homebrew formula with SHA-256"
    fi
}

# ── Main ───────────────────────────────────────────────────
usage() {
    echo "SANDBOX Package Builder v${SANDBOX_VERSION}"
    echo ""
    echo "Usage: $0 [target...]"
    echo ""
    echo "Targets:"
    echo "  all       Build all packages (default)"
    echo "  release   Build release binary only"
    echo "  deb       Build .deb package (Debian/Ubuntu)"
    echo "  rpm       Build .rpm package (Fedora/RHEL)"
    echo "  tarball   Build source tarball"
    echo "  clean     Remove build artifacts"
}

main() {
    local targets=("${@:-all}")

    echo ""
    echo -e "${BOLD}SANDBOX Package Builder v${SANDBOX_VERSION}${NC}"
    echo ""

    mkdir -p "$BUILD_DIR"

    for target in "${targets[@]}"; do
        case "$target" in
            all)
                build_release
                build_deb
                build_rpm
                build_tarball
                ;;
            release)  build_release ;;
            deb)      build_release && build_deb ;;
            rpm)      build_release && build_rpm ;;
            tarball)  build_release && build_tarball ;;
            clean)
                rm -rf "$BUILD_DIR"
                success "Cleaned build artifacts"
                ;;
            -h|--help|help)
                usage
                exit 0
                ;;
            *)
                error "Unknown target: $target (use --help)"
                ;;
        esac
    done

    echo ""
    info "Packages in: ${BUILD_DIR}/"
    ls -lh "${BUILD_DIR}"/*.{deb,rpm,tar.gz} 2>/dev/null || true
}

main "$@"
