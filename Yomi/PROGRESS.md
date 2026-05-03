# Yomi Progress Tracker

This file tracks the current implementation status of Yomi against the master specification in `context.md`.

## 🛠️ Core Engine
- [x] Project scaffolding (Rust 2021, Tokio)
- [x] File walker with `.gitignore` support (`ignore` crate)
- [x] BLAKE3 content hashing
- [x] Metadata extraction (size, mtime, language)
- [x] Binary file detection and skipping
- [ ] Symlink handling (configurable)
- [x] Basic file watching (`notify` crate)
- [ ] Incremental sync (debounce and atomic updates)

## 💾 Storage Layer
- [x] Basic JSON index storage
- [ ] Streaming JSON serialization (`serde_json::to_writer`) $\rightarrow$ **CRITICAL FIX**
- [ ] Migration to embedded DB (SQLite/Sled) $\rightarrow$ **Phase 3/4**

## 🔌 IPC & Interface
- [ ] Unix Socket server implementation
- [ ] JSON-RPC 2.0 protocol compliance
- [x] Basic CLI wrapper (`cargo run` commands)
- [ ] TUI implementation (`ratatui`) $\rightarrow$ **Phase 2**

## 🔍 Query Engine
- [ ] Path-based glob filtering
- [ ] Metadata-based filtering
- [ ] Raw content search (Ripgrep integration) $\rightarrow$ **Phase 3**
- [ ] Semantic search / BM25 scoring $\rightarrow$ **Parked (LLM will handle)**

## 📈 Performance & Stability
- [ ] I/O Throttling (`tokio::sync::Semaphore`) $\rightarrow$ **High Priority**
- [ ] UTF-8/Encoding robustness
- [ ] OS Watcher limit handling (fallback scan)

---

## 🎯 Immediate Priorities (The "Context Pipe" Roadmap)
1. **Stability**: Fix JSON OOM via streaming writers.
2. **Performance**: Implement I/O throttling for hashing.
3. **Robustness**: Improve encoding handling and watcher fallbacks.
4. **Interface**: Implement the UDS + JSON-RPC communication layer.
