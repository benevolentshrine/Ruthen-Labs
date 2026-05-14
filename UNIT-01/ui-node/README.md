# UNIT-01 Industrial UI

This is the next-generation Terminal User Interface (TUI) for UNIT-01, built with **Node.js**, **React**, and **Ink**.

## Why this exists
The previous UI was a static bash script. This new UI is a persistent application that allows for:
- **Fixed Layout**: A header that stays at the top of the terminal.
- **Dynamic Interactions**: Real-time streaming and interactive "Review Gates".
- **IPC Integration**: Direct communication with the Go Orchestrator, Rust Indexer, and Rust Sandbox.

## Setup
1. Ensure you have Node.js installed.
2. Install dependencies:
   ```bash
   npm install
   ```
3. Run in development mode:
   ```bash
   npm run dev
   ```

## Integration Map
- **Frontend**: Node.js / Ink
- **Orchestration**: Go (Backend logic)
- **Indexing**: Rust (Filesystem data)
- **Sandbox**: Rust (Secure execution)

## Industrial Aesthetics
- **Theme**: Amber/Grey/Black (Low-light, high-contrast).
- **Mode**: ARCHITECT (High-level reasoning).
- **Directives**: Tool-based execution via XML-style tags.
