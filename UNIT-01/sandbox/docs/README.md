# BORU 🥊

> **"What runs here, stays here."**

BORU is the **Security Engine** — the foundational layer of the [SumiLabs AI Ecosystem](https://github.com/ZenK-GH/SumiLabs).

Built in Rust. Powered by a deterministic 4-Layer Zero-Trust Sandbox. Zero network calls. Zero trust in AI-generated code until BORU says so.

---

## What BORU Does

BORU intercepts and sandboxes AI-generated code before it ever touches your system. It acts as a physical security membrane between your local LLM and your file system.

- 🔒 **Sandboxes** AI output via a 4-Layer Execution Jail (Pre-exec, Landlock, Seccomp, Cgroups).
- 🛡️ **Intercepts** unauthorized syscalls, destructive file access, and all network calls.
- 📜 **Logs** every blocked action with a tamper-proof SHA-256 audit trail.
- 🔌 **Runs** as a Unix socket daemon inside the SumiLabs ecosystem.
- 🖥️ **Works** standalone as a CLI/TUI tool.
- ⏪ **Rollback** filesystem changes made by AI agents.
- 🔍 **Scans** directories for threats with hash database matching.

---

## Architecture & Features

For a deep dive into how BORU works, check out the core documentation:

- 📖 **[BORU Architecture](ARCHITECTURE.md)**: Explains the 4-Layer Security Model and the SumiLabs Trinity Ecosystem.
- ⚙️ **[Feature Deep Dive](FEATURES.md)**: Detailed breakdown of the Unix Pipelines, Resource Quotas, and Static Gates.
- 📚 **[Docs Folder](docs/BORU_DOCUMENTATION.md)**: The single source of truth for CLI usage and socket integration contracts.

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

## Quickstart

### Installation

```bash
# Clone the repository
git clone https://github.com/ZenK-GH/SumiLabs.git
cd SumiLabs/boru

# Build from source (requires Rust)
cargo build --release

# Install the binary
sudo install -Dm755 target/release/boru /usr/local/bin/boru
```

### Basic Usage

**1. Sandboxing Code (Standalone Mode)**
Execute untrusted code inside the 4-Layer Sandbox:
```bash
# Basic execution with default security (mid)
boru cage --input target.py --mode mid

# Strict mode — maximum restrictions (zero-tolerance)
boru cage --input target.py --mode hard
```

**2. Daemon Mode (SumiLabs Ecosystem Integration)**
Run BORU as a background service listening on a Unix socket:
```bash
boru daemon &
```

**3. Visual Dashboard**
Launch the terminal UI to monitor live intercepts and executions:
```bash
boru tui
```

---

## Protocol Gates (Non-Negotiable Rules)

Before any code change is merged, all 7 Protocol Gates must pass. See [AGENTS.md](AGENTS.md).

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

## Name

**BORU** (ぼる) — Japanese for round, blocky, punchy. Exactly what a security cage should be.