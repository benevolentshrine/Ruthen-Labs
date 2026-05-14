# Suji: Sovereign AI Orchestrator

Suji is a high-performance, TUI-driven AI orchestrator designed to be the central brain of the MOMO ecosystem. It prioritizes local execution, security sandboxing, and structured context management.

## Features

- **TUI State Machine**: Robust handling of chat, tools, and reviews via `bubbletea`.
- **Dynamic Context**: Inject files via `@file` or search via the Yomi indexer.
- **Security Modes**: Hardware-enforced policy sync with the Boru sandbox.
- **MCP Support**: Connect to any Model Context Protocol server via stdio.
- **Memory Management**: Automatic token counting and context compaction.

## Setup

1. **Prerequisites**:
   - Go 1.21+
   - Ollama (running locally)
   - Boru & Yomi daemons (optional but recommended)

2. **Installation**:
   ```bash
   make install
   ```

3. **Configuration**:
   Configs are stored in `~/.config/suji/config.toml`.

## Commands

- `/search <q>`: Query the Yomi indexer.
- `/context`: Manage active files and tokens.
- `/models`: Switch Ollama models.
- `/style`: Toggle Casual/Context/Build modes.
- `/compact`: Manually summarize history.
- `/mcp`: Manage tool servers.

## Architecture

Suji uses a "Clean Room" orchestration model. It communicates with sidecar daemons via Unix Domain Sockets (UDS) using JSON-RPC 2.0, ensuring that the AI never has direct, ungated access to your host shell.
