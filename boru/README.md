# BORU 🥊

> **"What runs here, stays here."**

BORU is the **Security Cage** — the first engine of [Project MOMO](https://github.com/sayan5069/Momo.co), a local-first sovereign AI development suite.

Built in Rust. Powered by WASM sandboxing. Zero network calls. Zero trust in AI-generated code until BORU says so.

---

## What BORU Does

BORU intercepts and sandboxes AI-generated code before it ever touches your system. It acts as a security membrane between your local LLM and your file system.

- 🔒 **Sandboxes** AI output via WASM (`wasmtime`)
- 🛡️ **Intercepts** unauthorized syscalls, file access, and network calls
- 📜 **Logs** every blocked action with tamper-proof SHA-256 audit trail
- 🔌 **Runs** as a Unix socket daemon inside Project MOMO
- 🖥️ **Works** standalone as a CLI/TUI tool (Phase 1)
- ⏪ **Rollback** filesystem changes made by AI agents
- 🔍 **Scans** directories for threats with hash database matching

---

## Supported Platforms

| Platform | Status | Install Method |
|----------|--------|----------------|
| **Ubuntu / Debian / Mint / Pop!_OS** | ✅ Fully Supported | `.deb` package or source |
| **Fedora / RHEL / Rocky / Alma** | ✅ Fully Supported | `.rpm` package or source |
| **Arch / Manjaro / EndeavourOS** | ✅ Fully Supported | Source build |
| **openSUSE** | ✅ Fully Supported | `.rpm` package or source |
| **macOS (Intel & Apple Silicon)** | ✅ Fully Supported | Homebrew or source |
| **Windows** | ❌ Not Supported | Unix sockets are a core requirement |

---

## Installation

### Quick Install (All Platforms)

The universal installer auto-detects your OS and distribution:

```bash
git clone https://github.com/sayan5069/Momo.co.git
cd Momo.co/boru
./install.sh
```

### Debian / Ubuntu / Linux Mint / Pop!_OS

**From `.deb` package (recommended):**
```bash
# Download the latest .deb release
wget https://github.com/sayan5069/Momo.co/releases/latest/download/boru_0.3.0_amd64.deb

# Install
sudo dpkg -i boru_0.3.0_amd64.deb

# Fix any missing dependencies
sudo apt-get install -f
```

**From source:**
```bash
# Install build dependencies
sudo apt-get update
sudo apt-get install -y build-essential pkg-config curl

# Install Rust (if not present)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Clone and build
git clone https://github.com/sayan5069/Momo.co.git
cd Momo.co/boru
cargo build --release

# Install binary
sudo install -Dm755 target/release/boru /usr/local/bin/boru
```

### Fedora / RHEL / Rocky Linux / AlmaLinux

**From `.rpm` package (recommended):**
```bash
# Download the latest .rpm release
wget https://github.com/sayan5069/Momo.co/releases/latest/download/boru-0.3.0-1.x86_64.rpm

# Install (Fedora)
sudo dnf install ./boru-0.3.0-1.x86_64.rpm

# Or on RHEL/Rocky/Alma
sudo yum localinstall ./boru-0.3.0-1.x86_64.rpm
```

**From source:**
```bash
# Install build dependencies
sudo dnf install -y gcc gcc-c++ make pkg-config curl

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Clone and build
git clone https://github.com/sayan5069/Momo.co.git
cd Momo.co/boru
cargo build --release

# Install binary
sudo install -Dm755 target/release/boru /usr/local/bin/boru
```

### Arch Linux / Manjaro / EndeavourOS

```bash
# Install build dependencies
sudo pacman -S --needed base-devel rust

# Clone and build
git clone https://github.com/sayan5069/Momo.co.git
cd Momo.co/boru
cargo build --release

# Install binary
sudo install -Dm755 target/release/boru /usr/local/bin/boru
```

### openSUSE

```bash
# Install build dependencies
sudo zypper install -y -t pattern devel_basis
sudo zypper install -y cargo rust

# Clone and build
git clone https://github.com/sayan5069/Momo.co.git
cd Momo.co/boru
cargo build --release

# Install binary
sudo install -Dm755 target/release/boru /usr/local/bin/boru
```

### macOS (Homebrew)

```bash
# Using Homebrew (recommended)
brew tap sayan5069/momo
brew install boru

# Or from source
git clone https://github.com/sayan5069/Momo.co.git
cd Momo.co/boru
cargo build --release
sudo cp target/release/boru /usr/local/bin/
```

---

## Post-Install Setup

After installation, create the required runtime directories:

```bash
# Socket directory (auto-created by daemon, but you can pre-create)
sudo mkdir -p /tmp/momo
sudo chmod 0755 /tmp/momo

# Quarantine directory
sudo mkdir -p /tmp/momo/quarantine
sudo chmod 0700 /tmp/momo/quarantine

# Audit log directory
sudo mkdir -p /var/log/boru
sudo chmod 0755 /var/log/boru

# Hash database directory
sudo mkdir -p /var/lib/boru
sudo chmod 0755 /var/lib/boru
```

> **Note:** The `.deb` and `.rpm` packages handle this automatically via post-install scripts.

---

## How to Use BORU

After installing, verify BORU is available:

```bash
boru --version
# boru 0.3.0

boru --help
# Shows all available subcommands
```

---

### 1. Launching BORU (Daemon Mode)

BORU can run as a background daemon that listens on a Unix socket for execution requests from AI agents:

```bash
# Start the daemon in the background
boru daemon &

# The daemon creates and listens on /tmp/momo/boru.sock
# Other MOMO engines (SABA, ZUNO) communicate with BORU over this socket

# To use a custom socket path:
boru daemon --socket /path/to/custom.sock &
```

To stop the daemon:
```bash
# Find and kill the daemon process
pkill boru
# Or
kill $(pgrep boru)
```

---

### 2. Launching the TUI Dashboard

BORU includes a full terminal dashboard built with Ratatui for visual monitoring:

```bash
# Launch the TUI
boru tui

# Connect to a specific daemon socket
boru tui --socket /tmp/momo/boru.sock
```

> **Controls:** Navigate with arrow keys, press `q` to quit. The dashboard shows live audit events, system status, and intercept activity.

---

### 3. Sandboxing Code (The Core Feature)

This is BORU's heart — execute code inside a WASM sandbox:

```bash
# Basic execution with default security (mid)
boru cage --input target.wasm

# Strict mode — maximum restrictions
boru cage --input target.wasm --mode hard

# Available security modes:
#   hard   — Maximum restrictions, minimal permissions
#   mid    — Balanced (default) — blocks dangerous ops, allows safe ones
#   easy   — Permissive — logs everything but blocks less
#   audit  — Log-only mode — nothing is blocked, everything is recorded
#   custom — Load rules from a config file

# Set a fuel limit (max WASM instructions before timeout)
boru cage --input target.wasm --mode hard --fuel 1000000

# Use a custom security config
boru cage --input target.wasm --mode custom --config ./my-rules.toml
```

**Output:** BORU returns a verdict — `ALLOWED` or `BLOCKED` — along with details:
```
Verdict: BLOCKED
Reason: Unauthorized file system write to /etc/passwd
Mode: hard
Fuel used: 42,389 / 1,000,000
Audit ref: #00127
```

**Exit codes:**
| Code | Meaning |
|------|---------|
| `0` | Execution allowed |
| `1` | Execution blocked |
| `2` | Error (invalid input, config failure) |
| `3` | Timeout (fuel limit exceeded) |

---

### 4. Static Analysis (Dry Run)

Analyze a file without executing it:

```bash
boru check --input suspicious_file.wasm
```

This performs static analysis — inspects the binary for dangerous patterns without running it.

---

### 5. Directory Scanning

Scan entire directories for threats:

```bash
# Basic scan
boru scan --path ./my-project/

# Strict mode scan
boru scan --path ./my-project/ --mode hard

# Limit recursion depth
boru scan --path ./my-project/ --depth 3

# Export report as Markdown
boru scan --path ./my-project/ --format markdown --output report.md

# Console output (default)
boru scan --path ./my-project/ --format console
```

---

### 6. Real-Time File Monitoring (Watchdog)

Watch a directory for changes and auto-scan new/modified files:

```bash
# Start watching a directory
boru watch --path ./my-project/

# Watch with strict security mode
boru watch --path ./my-project/ --mode hard

# Non-recursive (only top-level)
boru watch --path ./my-project/ --recursive false

# Use polling fallback (if native file events aren't working)
boru watch --path ./my-project/ --poll
```

Press `Ctrl+C` to stop the watchdog. Events are printed live:
```
🔍 BORU Watchdog — watching ./my-project/ (mid mode)
📄 New file: ./my-project/script.py
✏️  Modified: ./my-project/main.rs
   ✅ ./my-project/main.rs (entropy: 4.2)
🔴 QUARANTINED: ./my-project/payload.bin — High entropy binary detected
```

---

### 7. Audit Logs

Every intercept, block, and execution is logged with a tamper-proof SHA-256 chain:

```bash
# View recent logs
boru log

# Live-tail logs (like tail -f)
boru log --tail

# Filter by severity
boru log --severity CRITICAL
boru log --severity HIGH

# View logs since a timestamp
boru log --since "2026-04-24T00:00:00"

# Export logs to a file
boru log --export ./audit-export.json

# Verify the entire tamper chain integrity
boru log --verify
# Output: "Tamper chain verified: 127 entries, 0 anomalies"

# Verify a specific entry
boru log --verify --entry 42

# Clear all logs (with confirmation)
boru log --clear
```

---

### 8. Quarantine Management

Files flagged as malicious are moved to quarantine (`/tmp/momo/quarantine/`):

```bash
# List all quarantined files
boru quarantine --list

# Restore a quarantined file to its original location
boru quarantine --restore <ID>

# Permanently delete a quarantined file
boru quarantine --delete <ID>
```

---

### 9. Threat Hash Database

BORU maintains a local database of SHA-256 hashes for known-bad files:

```bash
# Check a file against the database
boru hash --check ./suspicious_file.wasm

# Add a known-bad hash
boru hash --add "abc123...sha256..." --name "trojan-loader" --family "wasm-malware"

# Remove a hash
boru hash --remove "abc123...sha256..."

# Import hashes from a JSON file
boru hash --import ./threat-hashes.json

# List all entries
boru hash --list

# Show database statistics
boru hash --stats
```

---

### 10. Session Replay (Forensics)

BORU records execution timelines for debugging and forensic analysis:

```bash
# List all recorded sessions
boru replay --list

# Replay a specific session timeline
boru replay --session <SESSION_ID>

# Export session data to a file
boru replay --session <SESSION_ID> --export ./session-dump.json
```

---

### 11. Filesystem Rollback (Shadow Snapshots)

BORU captures filesystem state before AI execution, allowing you to undo destructive changes:

```bash
# List all shadow backups
boru rollback --list

# Preview what would be restored (dry run)
boru rollback --session <SESSION_ID> --dry-run

# Execute the rollback
boru rollback --session <SESSION_ID>

# Clear shadow data for a session
boru rollback --clear <SESSION_ID>
```

---

### 12. Agent Identity Management (IAM)

Manage tokens for AI agents that connect over the socket:

```bash
# Create a new agent and receive a token
boru iam --create-agent "claude-agent" --description "Claude Code assistant"
# ╔══════════════════════════════════════════════════╗
# ║  SAVE THIS TOKEN — IT WILL NOT BE SHOWN AGAIN   ║
# ╠══════════════════════════════════════════════════╣
# ║  boru_tk_a1b2c3d4e5f6...                        ║
# ╚══════════════════════════════════════════════════╝

# List all registered agents
boru iam --list

# Show details for a specific agent
boru iam --show "claude-agent"

# Revoke an agent's access
boru iam --revoke "claude-agent"
```

---

### 13. Dependency Check

Check which runtime interpreters are available on your system:

```bash
boru deps
# BORU Dependency Status
# =======================
# WasmRunner:
#   ✓ wasmtime (25.0.0) at /usr/local/bin/wasmtime
# PythonRunner:
#   ✓ python3 (3.11.2) at /usr/bin/python3
# NodeRunner:
#   ✗ node (not found)
```

---

### Common Workflows

#### Workflow A: One-shot sandbox (simplest use case)
```bash
boru cage --input untrusted.wasm --mode hard --fuel 500000
```

#### Workflow B: Always-on monitoring
```bash
boru daemon &                          # Start background daemon
boru watch --path ~/code/ --mode mid   # Watch your code directory
# In another terminal:
boru tui                               # Visual dashboard
```

#### Workflow C: Incident response
```bash
boru log --severity CRITICAL           # Check what happened
boru replay --session <ID>             # Replay the timeline
boru rollback --session <ID> --dry-run # Preview restoration
boru rollback --session <ID>           # Undo the damage
```

---

## Architecture

```
MOMO Suite (Trinity Architecture)
├── BORU  ← You are here (Security Cage)     /tmp/momo/boru.sock
├── ZUNO  ← Memory / Indexer (planned)       /tmp/momo/zuno.sock
└── SABA  ← Router / Nervous System (planned) /tmp/momo/saba.sock
```

| Phase | Description | Status |
|-------|-------------|--------|
| 1 | Standalone CLI/TUI tool | 🔨 Active |
| 2 | MOMO-integrated daemon with GUI panel | 📋 Planned |

---

## Building Packages

BORU includes packaging infrastructure for creating distribution packages:

```bash
# Build all packages (.deb, .rpm, tarball)
./packaging/build-packages.sh all

# Build only .deb
./packaging/build-packages.sh deb

# Build only .rpm
./packaging/build-packages.sh rpm

# Build source tarball (for Homebrew)
./packaging/build-packages.sh tarball
```

---

## Verification

```bash
# Check binary size (must stay < 10MB — Gate 1)
ls -lh target/release/boru

# Check idle memory (must stay < 20MB — Gate 6)
boru daemon &
sleep 2
ps aux | grep boru | awk '{print $6}'  # RSS in KB, must be < 20480
```

---

## Protocol Gates (Non-Negotiable Rules)

Before any code change, all 7 Protocol Gates must pass. See [AGENTS.md](AGENTS.md).

| Gate | Rule | Budget |
|------|------|--------|
| 1 | Zero Bloat Law | Binary < 10MB |
| 2 | Sandbox Invariant | All exec via `src/cage/` |
| 3 | Socket Contract Freeze | Paths are hardcoded |
| 4 | No Network Calls | Zero network crates |
| 5 | Phase Lock | No GUI deps in Phase 1 |
| 6 | Memory Budget | Idle RAM < 20MB |
| 7 | Intercept Audit Log | Every block is logged |

---

## Project MOMO

BORU is one part of a larger vision. See [ARCHITECTURE.md](ARCHITECTURE.md) for the full Trinity design.

---

## Name

**BORU** (ぼる) — Japanese for round, blocky, punchy. Exactly what a security cage should be.