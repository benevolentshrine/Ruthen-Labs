# Inspiration & conceptual Borrowings

Suji stands on the shoulders of giants in the open-source community. This log documents the conceptual and architectural borrowings that shaped its design.

## Architecture

- **Bubble Tea (Charmbracelet)**: The state-driven TUI model is inspired by the Elm Architecture and the excellent `bubbletea` framework.
- **MCP (Anthropic)**: The Model Context Protocol implementation for tool interoperability.
- **Ollama**: The streaming NDJSON protocol for local LLM inference.

## Licensing References

- **Apache 2.0**: The overall project structure and modularity goal adhere to the principles of openness and commercial friendliness found in Apache-licensed projects.
- **Clean Room Implementation**: All code in this repository was written from scratch to ensure a clear legal pedigree, avoiding direct derivation from existing agentic tools.

## Conceptual Nodes

- **The "Boru" Concept**: Inspired by the need for secure, capability-based sandboxing in agentic loops.
- **Yomi Indexing**: A conceptual nod to high-performance vector search and local context retrieval patterns.
