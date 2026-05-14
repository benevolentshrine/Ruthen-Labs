# SANDBOX Features Deep Dive

This document details the core features and internal pipelines of the SANDBOX Security Engine.

---

## 1. The Unix I/O Pipeline (Execution Isolation)

When SANDBOX executes a script, it cannot simply pipe the script's `stdout` directly to the host terminal, as this risks injection attacks and mixes internal security logs with user output.

Instead, SANDBOX uses a strict **I/O Pipeline**:
1. **Process Forking**: The child process is spawned via Rust's `Command::new()`.
2. **Environment Stripping**: `env_clear()` is called immediately to wipe all host environment variables, preventing the script from reading sensitive tokens (like `$AWS_ACCESS_KEY_ID`).
3. **Piped Streams**: `.stdout(Stdio::piped())` and `.stderr(Stdio::piped())` are attached. The child process cannot write directly to the parent's TTY.
4. **Log Silencing**: The child's environment is injected with `RUST_LOG=off`. This ensures that if the child itself is a Rust binary (or another SANDBOX instance), it won't dump internal tracing logs.
5. **Output Filtering**: Once the child process exits, SANDBOX captures the raw `stdout`. It applies a string filter to remove any internal kernel/seccomp warnings (`sandbox::cage::sandbox`, `INFO sandbox::`, etc.) that were emitted by the pre-exec closure before the child executed. 
6. **Clean Return**: The sanitized string is returned as the final output over the Unix socket.

---

## 2. Kernel-Level Isolation (Seccomp + Landlock)

SANDBOX v2.0 implements dual-layered kernel enforcement that cannot be bypassed from user-space, even if the script exploits a vulnerability in the Python interpreter.

### Landlock ABI v2 (Filesystem Jail)
*   **Zero-Visibility**: By default, the entire filesystem tree (`/`, `/home`, `/etc`) is inaccessible. `stat()` calls will return `ENOENT` (No such file or directory) or `EACCES` (Permission denied).
*   **Targeted Allowlist**: SANDBOX selectively exposes:
    *   `/tmp/ruthenlabs/workspace` (Read/Write)
    *   `/usr/bin` (Read-Only)
    *   `/usr/share` & `/usr/local/lib` (Read-Only for language standard libraries)
    *   `/proc/self` (Read-Only for runtime introspection)

### Seccomp-BPF v2 (Network Air-Gap)
*   **The Filter**: A Berkeley Packet Filter (BPF) program is compiled into the kernel for the child process.
*   **Hard Blocks**: Any attempt to call `socket()`, `connect()`, or `bind()` results in an immediate kernel-level `SIGSYS` kill. The process is terminated instantly before the operation executes.
*   **Process Protection**: `clone3`, `unshare`, and `ptrace` are blocked to prevent the sandboxed script from creating new namespaces or debugging the parent SANDBOX process.

---

## 3. Pre-Execution Static Gate

Before delegating enforcement to the kernel, SANDBOX performs a static text analysis of the source code. This acts as an immediate fail-safe.

*   **HARD Mode (Zero-Tolerance)**: Scans for and blocks:
    *   `import os`, `import subprocess`, `import socket`
    *   `open("/etc/`
    *   Any system command invocation (`os.system`)
*   **MID Mode (Production Default)**: Scans for and blocks destructive operations that could permanently damage the host if the kernel sandbox degrades:
    *   `os.remove(`, `os.unlink(`, `os.rmdir(`
    *   `shutil.rmtree(`, `shutil.move(`
    *   `os.environ`, `os.getenv(` (Prevents environment variable exfiltration)

If any pattern matches, SANDBOX aborts the execution *before* `fork()` is even called, returning a `BLOCKED` verdict.

---

## 4. Resource Quotas (Cgroups v2)

To prevent an LLM from accidentally (or maliciously) generating code that crashes the host machine (e.g., a memory leak or a fork bomb), SANDBOX utilizes Linux Cgroups v2.

*   **User Slice Delegation**: SANDBOX runs unprivileged. It places child processes into `/sys/fs/cgroup/user.slice/user-1000.service/app.slice`, utilizing systemd's user delegation to manage resources without `sudo`.
*   **Memory Ceiling**: The `memory.max` controller is configured. If the script allocates more memory than the budget, the kernel OOM-killer targets the sandbox, not the host.
*   **PID Limits**: The `pids.max` controller is configured (default 20) to prevent the script from spawning infinite background threads.

---

## 5. Tamper-Proof Audit Logging

Every execution, whether allowed or blocked, generates an audit log entry.
*   **Structure**: Logs are stored locally and contain the Timestamp, Severity, Action Type, and detailed Reason.
*   **Immutability**: Log entries are chained together using SHA-256 hashes (similar to a blockchain). If a malicious agent somehow escapes the sandbox and edits a past log entry, the hash chain will break, and the SANDBOX daemon will detect the tamper upon the next startup.

---

## 6. Daemon Mode & TUI Dashboard

SANDBOX is not just a CLI execution tool; it is a long-running service.
*   **Unix Socket Server**: Runs in the background and accepts asynchronous JSON payloads.
*   **Ratatui Dashboard**: A terminal-based user interface that connects to the running daemon. It displays a live stream of execution events, the status of the security gates, and any intercepted network/filesystem calls in real-time.
