%define _name sandbox
%define _version 0.3.0
%define _release 1

Name:           %{_name}
Version:        %{_version}
Release:        %{_release}%{?dist}
Summary:        Security Cage engine for AI-generated code — Project RUTHENLABS
License:        Apache-2.0
URL:            https://github.com/sayan5069/RuthenLabs.co
Source0:        %{_name}-%{_version}.tar.gz

BuildRequires:  cargo >= 1.75
BuildRequires:  rust >= 1.75
BuildRequires:  gcc
BuildRequires:  make

# SANDBOX runs exclusively on Unix-like systems (Linux + macOS)
ExclusiveArch:  x86_64 aarch64

%description
SANDBOX is the Security Cage engine of Project RUTHENLABS, a local-first sovereign
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
install -Dm755 target/release/sandbox %{buildroot}%{_bindir}/sandbox

# Man page (if exists)
if [ -f docs/sandbox.1 ]; then
    install -Dm644 docs/sandbox.1 %{buildroot}%{_mandir}/man1/sandbox.1
fi

# Create runtime directories
install -dm755 %{buildroot}/tmp/ruthenlabs
install -dm700 %{buildroot}/tmp/ruthenlabs/quarantine

%post
# Create runtime directories if they don't exist
if [ ! -d "/tmp/ruthenlabs" ]; then
    mkdir -p /tmp/ruthenlabs
    chmod 0755 /tmp/ruthenlabs
fi

if [ ! -d "/tmp/ruthenlabs/quarantine" ]; then
    mkdir -p /tmp/ruthenlabs/quarantine
    chmod 0700 /tmp/ruthenlabs/quarantine
fi

if [ ! -d "/var/log/sandbox" ]; then
    mkdir -p /var/log/sandbox
    chmod 0755 /var/log/sandbox
fi

if [ ! -d "/var/lib/sandbox" ]; then
    mkdir -p /var/lib/sandbox
    chmod 0755 /var/lib/sandbox
fi

echo ""
echo "  ╔═══════════════════════════════════════════╗"
echo "  ║   SANDBOX installed successfully 🥊          ║"
echo "  ║                                           ║"
echo "  ║   Run 'sandbox daemon' to start the server   ║"
echo "  ║   Run 'sandbox tui' for the dashboard        ║"
echo "  ╚═══════════════════════════════════════════╝"
echo ""

%postun
if [ "$1" = 0 ]; then
    # Full removal — clean up socket only (preserve user data)
    rm -f /tmp/ruthenlabs/sandbox.sock 2>/dev/null || true
fi

%files
%license LICENSE
%doc README.md ARCHITECTURE.md AGENTS.md CHANGELOG.md
%{_bindir}/sandbox

%changelog
* Thu Apr 24 2026 SANDBOX Team <sandbox@projectruthenlabs.dev> - 0.3.0-1
- Initial RPM package release
- WASM sandboxing with wasmtime
- TUI dashboard (ratatui)
- Audit logging with tamper-proof SHA-256 chain
- Quarantine and rollback capabilities
- Hash database for threat detection
- Agent IAM system
- Scanner and watchdog modules
