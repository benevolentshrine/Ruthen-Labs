%define _name boru
%define _version 0.3.0
%define _release 1

Name:           %{_name}
Version:        %{_version}
Release:        %{_release}%{?dist}
Summary:        Security Cage engine for AI-generated code — Project MOMO
License:        Apache-2.0
URL:            https://github.com/sayan5069/Momo.co
Source0:        %{_name}-%{_version}.tar.gz

BuildRequires:  cargo >= 1.75
BuildRequires:  rust >= 1.75
BuildRequires:  gcc
BuildRequires:  make

# BORU runs exclusively on Unix-like systems (Linux + macOS)
ExclusiveArch:  x86_64 aarch64

%description
BORU is the Security Cage engine of Project MOMO, a local-first sovereign
AI development suite. It intercepts and sandboxes AI-generated code before
it touches your system using WebAssembly (wasmtime) sandboxing.

Features:
- WASM-based sandboxing for AI-generated code
- Syscall, file, and network interception
- Tamper-proof audit logging with SHA-256 chain
- TUI dashboard for monitoring (ratatui)
- Quarantine and rollback capabilities
- Agent identity management (IAM)
- Zero network calls — fully offline operation

"What runs here, stays here."

%prep
%setup -q -n %{_name}-%{_version}

%build
cargo build --release

%install
# Binary
install -Dm755 target/release/boru %{buildroot}%{_bindir}/boru

# Man page (if exists)
if [ -f docs/boru.1 ]; then
    install -Dm644 docs/boru.1 %{buildroot}%{_mandir}/man1/boru.1
fi

# Create runtime directories
install -dm755 %{buildroot}/tmp/momo
install -dm700 %{buildroot}/tmp/momo/quarantine

%post
# Create runtime directories if they don't exist
if [ ! -d "/tmp/momo" ]; then
    mkdir -p /tmp/momo
    chmod 0755 /tmp/momo
fi

if [ ! -d "/tmp/momo/quarantine" ]; then
    mkdir -p /tmp/momo/quarantine
    chmod 0700 /tmp/momo/quarantine
fi

if [ ! -d "/var/log/boru" ]; then
    mkdir -p /var/log/boru
    chmod 0755 /var/log/boru
fi

if [ ! -d "/var/lib/boru" ]; then
    mkdir -p /var/lib/boru
    chmod 0755 /var/lib/boru
fi

echo ""
echo "  ╔═══════════════════════════════════════════╗"
echo "  ║   BORU installed successfully 🥊          ║"
echo "  ║                                           ║"
echo "  ║   Run 'boru daemon' to start the server   ║"
echo "  ║   Run 'boru tui' for the dashboard        ║"
echo "  ╚═══════════════════════════════════════════╝"
echo ""

%postun
if [ "$1" = 0 ]; then
    # Full removal — clean up socket only (preserve user data)
    rm -f /tmp/momo/boru.sock 2>/dev/null || true
fi

%files
%license LICENSE
%doc README.md ARCHITECTURE.md AGENTS.md CHANGELOG.md
%{_bindir}/boru

%changelog
* Thu Apr 24 2026 BORU Team <boru@projectmomo.dev> - 0.3.0-1
- Initial RPM package release
- WASM sandboxing with wasmtime
- TUI dashboard (ratatui)
- Audit logging with tamper-proof SHA-256 chain
- Quarantine and rollback capabilities
- Hash database for threat detection
- Agent IAM system
- Scanner and watchdog modules
