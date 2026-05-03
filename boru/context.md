# BORU Master Context Document

This document serves as the absolute source of truth for the entire BORU project history, architecture, versions, constraints, and known bug fixes. It is designed to rapidly onboard developers (and AI agents) with the exact state of the project.

---

## 1. Project Overview & Trinity Architecture

**BORU** is the foundational "Security Engine" of the **SumiLabs AI Ecosystem** (formerly Project MOMO). It is a deterministic, multi-layered sandbox designed to safely execute untrusted, AI-generated code.

The ecosystem uses a "Trinity" architecture communicating exclusively via **Unix Domain Sockets (UDS) + JSON-RPC** over `$TMPDIR/momo/*.sock`:
1. **BORU (Security Cage)**: Executes code in a hardware-enforced jail.
2. **SUJI (Router/Nervous System)**: The AI agent orchestrator.
3. **YOMI**: Indexing, context retrieval, and memory modules.

---

## 2. The 7 Protocol Gates (Non-Negotiable Constraints)

BORU enforces extreme efficiency and security constraints:
1. **Zero Bloat Law**: The release binary must remain `< 10MB`. (No heavy frameworks like `tokio` or `reqwest`).
2. **Sandbox Invariant**: All execution must happen via `src/cage/`. Host execution is strictly forbidden.
3. **Socket Contract Freeze**: Inter-process communication uses hardcoded Unix Socket paths relative to `$TMPDIR`.
4. **No Network Calls**: Zero network crates. Completely air-gapped.
5. **Phase Lock**: No GUI dependencies inside the daemon phase.
6. **Memory Budget**: Idle RAM footprint must remain `< 20MB`.
7. **Intercept Audit Log**: Every block/execution must be hashed and logged.

---

## 3. Versions and Phases

### Phase 1: Standalone CLI (v0.1.0 - v0.2.0)
- **Initial Concept**: Relied heavily on `wasmtime` to sandbox WebAssembly binaries. 
- **Tooling**: Included static analysis, directory scanning, and the Ratatui-based TUI dashboard.

### Phase 1.2: The Hard Linux Cage (v0.3.0 / Engine v2.0)
- **Evolution**: Shifted from purely WASM to natively supporting Python/Bash via deep kernel primitives.
- **Current State**: The engine now enforces a strict **4-Layer Physical Containment Boundary**.

---

## 4. The 4-Layer Zero-Trust Sandbox (v2.0)

When SUJI asks BORU to execute code, the following enforcement layers are applied:

1. **Pre-Execution Static Gate**: Parses source text before `fork()`.
   The strictness is controlled by the active **Security Mode**:
   - *HARD Mode*: Zero-tolerance. Blocks `import os`, `subprocess`, `/etc/`, and any network module.
   - *MID Mode* (Default): Blocks destructive file operations (`os.remove`, `shutil.rmtree`) and environment access (`os.environ`). Allows basic standard library access.
   - *EASY Mode*: Permissive. Blocks very little, but logs all actions.
   - *AUDIT Mode*: Log-only mode. Nothing is blocked, everything is recorded for forensics.
   - *(CUSTOM Mode)*: Allows loading a specific rule set via a `.toml` config.
2. **Landlock ABI v2 (Filesystem Jail)**:
   - Sets a default-deny on the entire OS (`/`).
   - Allowlists `/tmp/momo/workspace` for Read/Write.
   - Allowlists `/usr/bin`, `/usr/share`, `/usr/local/lib`, and `/proc/self` as Read-Only so interpreters can boot.
3. **Seccomp-BPF v2 (Network Air-Gap)**:
   - Injects a Berkeley Packet Filter into the kernel.
   - Blocks `socket()`, `connect()`, `bind()`, `clone3`, `unshare`, and `ptrace` with immediate `SIGSYS` death.
4. **Cgroups v2 (Resource Quotas)**:
   - Limits RAM (e.g., 512MB) and PIDs (e.g., 20 max threads) to prevent OOM/Fork-bombs.
   - Runs in the unprivileged `user.slice/user-1000.slice/user@1000.service/app.slice` to avoid needing `sudo`.

---

## 5. Major Bugs, Errors, and Fixes (Development History)

During the development of the v2.0 Hard Linux Cage, several critical issues were encountered and permanently fixed:

### Bug 1: Policy Engine Disconnect (The `/etc/passwd` Leak)
* **Error**: Running `boru cage --input target.py --mode hard` allowed Python to freely read `/etc/passwd` despite the `hard` flag.
* **Root Cause**: The static analysis gate was bypassed, and Landlock wasn't enforcing strict enough rules on interpreters.
* **The Fix**: Extended the `MID`/`HARD` pattern scanners to intercept Python destructive file ops. Landlock was tightened to strictly whitelist only the ephemeral workspace.

### Bug 2: Process Logs Polluting Execution Output
* **Error**: Internal `boru::cage::sandbox` tracing logs (like "Landlock applied") were mixing with the actual `stdout` of the user's Python script, causing SUJI/agents to hallucinate.
* **Root Cause**: The parent process was writing tracing logs to `stderr`, and the child process inherited the parent's TTY.
* **The Fix**: 
  1. The Unix I/O Pipeline was overhauled.
  2. `sandbox_cmd.stdout(Stdio::piped()).stderr(Stdio::piped())` was added.
  3. The child environment is wiped (`env_clear()`) and forced to `RUST_LOG=off`.
  4. BORU captures raw `stdout` and applies a string filter to strip out any remaining `boru::` kernel warnings before returning the payload over the socket.

### Bug 3: Seccomp Crashing Python 3.12 (`SIGSYS`)
* **Error**: Python scripts immediately crashed with `Bad system call` when Seccomp was enabled.
* **Root Cause**: The Seccomp allowlist was too restrictive for modern Python interpreters, which require dynamic linking and random number generation to boot.
* **The Fix**: Expanded the allowlist to include: `getrandom`, `prlimit64`, `ioctl`, `newfstatat`, `fcntl`, `clone`, `wait4`, `prctl`, `capget`, `sched_getaffinity`, `gettid`, `set_robust_list`, and `execve`. 

### Bug 4: Landlock ABI Permission Denied for Interpreters
* **Error**: Running python failed because Landlock blocked access to standard libraries.
* **Root Cause**: Landlock default-deny blocked `/usr/bin/python3` from reading `/usr/lib/python3.12`.
* **The Fix**: Explicitly whitelisted `/usr/bin`, `/usr/share`, `/usr/local/lib`, and `/proc/self` with `ACCESS_FS_READ_FILE | ACCESS_FS_READ_DIR`.

### Bug 5: Cgroups Requiring `sudo`
* **Error**: Writing to `/sys/fs/cgroup/` directly threw a `Permission Denied` error for non-root users.
* **Root Cause**: Modifying the root cgroup hierarchy requires root escalation, breaking the goal of an unprivileged daemon.
* **The Fix**: Re-routed the `CGROUP_ROOT` constant to target systemd's user delegation: `/sys/fs/cgroup/user.slice/user-1000.slice/user@1000.service/app.slice`.

### Codebase Hardening: AI Comment & Dead Code Purge
* **Issue**: The codebase contained "tutorial-like" comments (e.g., `// Step 1: Do this`) left by LLM code generators, alongside dead code flagged by `cargo clippy`.
* **The Fix**: A massive security audit scrubbed all leaked API keys (none found), removed unused `HashStatus::Unknown` states, stripped unused imports (`cargo clippy --fix`), and converted all LLM comments into idiomatic, professional Rust docstrings.

---

## 6. Current Implementation Goals

1. **UDS + JSON-RPC Integration**: 
   - Finalizing the socket contract to use strict JSON-RPC 2.0 payloads for SUJI-to-BORU communication.
2. **Seccomp User Notification Layer (Phase 2)**: 
   - Planning to implement `SECCOMP_RET_USER_NOTIF` to allow the TUI dashboard to dynamically prompt the user if a script requests a permissive system call.
