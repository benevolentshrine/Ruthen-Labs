# SumiLabs

> **"What runs here, stays here."**

## Description
A sovereign, local-first AI development environment and infrastructure replacing cloud wrappers. Built for privacy, speed, and absolute control over your AI agents and tools.

## Architecture

| Component | Technology | Description |
|-----------|------------|-------------|
| **Boru** | Rust/Sandbox | Security Sandbox for executing AI-generated or untrusted code safely. |
| **Yomi** | Rust/Indexer | Ultra-fast Codebase Indexer for parsing and managing local workspace knowledge. |
| **Suji** | Go/Orchestrator | AI Orchestrator that coordinates agents, tasks, and system interactions. |
| **Momo GUI** | Tauri/React | The overarching graphical user interface unifying the entire ecosystem. |

## Quickstart

### Prerequisites
- Rust (`rustup default stable`)
- Go 1.21+
- Node.js & npm/yarn (for Tauri/React GUI)

### Setup
1. **Clone the repository:**
   ```bash
   git clone <your-repo-url>
   cd SumiLabs
   ```
2. **Review contribution guidelines:**
   Before making changes, please read [CONTRIBUTING.md](./CONTRIBUTING.md) to understand our strict DCO and PR rules.
3. **Run CI locally:**
   - **Rust modules (`boru`, `yomi`):**
     ```bash
     cd <module-name>
     cargo fmt --all -- --check
     cargo clippy -- -D warnings
     cargo test
     ```
   - **Go modules (`suji`):**
     ```bash
     cd suji
     go fmt ./...
     go vet ./...
     go test -v ./...
     ```

## License
Licensed under the [Apache License 2.0](./LICENSE). See the LICENSE file for details.
