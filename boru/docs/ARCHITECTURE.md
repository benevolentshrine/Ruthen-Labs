# BORU Architecture v2.0

## SumiLabs Trinity Overview

The SumiLabs AI Ecosystem is powered by three localized engines communicating over Unix domain sockets. No network calls. No cloud leaks. Everything stays on the host.

```
UI / CLI
   │
   └──► SABA  (Router / Agent)     $TMPDIR/momo/saba.sock   [NOT YET IMPLEMENTED]
           │
           ├──► ZUNO  (Indexer)    $TMPDIR/momo/zuno.sock   [NOT YET IMPLEMENTED]
           │         context retrieval
           │
           └──► BORU  (Security)   $TMPDIR/momo/boru.sock   [ACTIVE]
                       sandboxed execution
                            │
                            └──► Local LLM (Ollama / Llama.cpp)
```

---

## BORU's Role

BORU is the **Security Engine**. Every piece of AI-generated code that needs execution must pass through BORU first. 

### Responsibilities
- Accept execution requests over the Unix socket.
- Evaluate the source code via static analysis before spawning anything.
- Confine execution using a strict 4-Layer Kernel Sandbox.
- Enforce resource quotas (RAM, CPU, PIDs).
- Capture filtered output (stdout/stderr) cleanly.
- Return: `ALLOWED` or `BLOCKED` + a secure audit log entry.
- **Never pass execution to the host OS without sandbox clearance.**

### What BORU Does NOT Do
- Route requests (that is SABA's job).
- Index the user's workspace (that is ZUNO's job).
- Talk to the LLM directly.
- Make any network calls.

---

## The 4-Layer Sandbox Architecture

BORU v2.0 implements a deterministic, multi-layered physical containment boundary.

### 1. Pre-Execution Static Gate (The Source Layer)
Before `fork()` is even called, BORU parses the raw source code text and evaluates it against the requested `SecurityMode`.
- **HARD**: Fails fast if any `import os`, `/etc/passwd`, or network-related patterns are found.
- **MID**: Blocks destructive operations like `os.remove` and `shutil.rmtree`.

### 2. Landlock ABI v2 (The Filesystem Jail)
Once the child process is forked (but before it executes), BORU applies an unprivileged Landlock filesystem jail.
- **Strict Allowlist**: Only the ephemeral workspace is granted Read/Write access.
- **System Isolation**: Interpreters (like `/usr/bin/python3` or `/lib`) are granted Read-Only access. Everything else on the host machine is completely invisible to the child process.

### 3. Seccomp-BPF v2 (The Kernel Layer)
A strict Berkeley Packet Filter (BPF) policy is loaded to intercept System Calls.
- **Network Air-Gap**: Syscalls like `socket()`, `bind()`, and `connect()` trigger an immediate `SIGSYS` kernel termination.
- **Privilege Drop**: `clone3`, `ptrace`, and `unshare` are blocked to prevent container escapes.

### 4. Cgroups v2 (The Resource Layer)
To prevent Denial of Service attacks (like infinite loops or fork bombs), BORU delegates the child process into a strict systemd user slice (`user@1000.service/app.slice`).
- **Memory Ceiling**: Hard limit enforced (e.g., 512MB) before the OOM killer triggers.
- **PID Limit**: Caps the maximum number of threads/processes.

---

## Socket Contract

BORU resolves its socket dynamically based on the OS temporary directory (e.g., `$TMPDIR/momo/boru.sock`). 

**Request format (JSON over Unix socket):**
```json
{
  "request_id": "uuid-v4",
  "type": "execute",
  "payload": {
    "code": "base64-encoded python/wasm",
    "format": "python | wasm",
    "policy": "strict | permissive"
  }
}
```

**Response format:**
```json
{
  "request_id": "uuid-v4",
  "verdict": "ALLOWED | BLOCKED",
  "reason": "string or null",
  "output": "stdout captured cleanly without tracing logs",
  "audit_ref": "log entry id"
}
```

---

## Binary Budget

BORU adheres strictly to the **Zero Bloat Law** to ensure it remains a lightweight daemon.

| Engine | RAM Budget | Binary Size |
|--------|-----------|-------------|
| BORU   | < 20MB idle RSS | < 10MB release |
| ZUNO   | < 15MB (target) | TBD |
| SABA   | < 25MB (target) | TBD |

---

## Security Invariant

> **BORU is the last line of defense. Nothing executes without cage clearance.**

This invariant must never be broken. If the LLM orchestrator bypasses BORU for "performance" or "convenience," the entire SumiLabs security model collapses.