//! BORU Socket Stubs — Reserved for YOMI and SUJI
//!
//! These are placeholder stubs for the Trinity architecture.
//! DO NOT implement YOMI or SUJI logic here — that lives in their own repositories.
//!
//! GATE 3: Socket Contract Freeze
//! - YOMI socket: [TEMP]/momo/yomi.sock
//! - SUJI socket: [TEMP]/momo/suji.sock
//! - BORU socket: [TEMP]/momo/boru.sock (active)

#![allow(dead_code)]

use std::path::Path;
use crate::socket::config::{boru_socket_path, suji_socket_path, yomi_socket_path};

/// Stub function for YOMI socket operations
///
/// YOMI is the Rust indexer — context retrieval engine.
/// BORU never calls YOMI directly. YOMI may call BORU.
pub fn yomi_stub() -> anyhow::Result<()> {
    // This is intentionally a stub.
    // YOMI implementation lives in the YOMI repository.
    Ok(())
}

/// Stub function for SUJI socket operations
///
/// SUJI is the Go router — request orchestration layer.
/// BORU never calls SUJI. SUJI calls BORU.
pub fn suji_stub() -> anyhow::Result<()> {
    // This is intentionally a stub.
    // SUJI implementation lives in the SUJI repository.
    Ok(())
}

/// Validate that a socket path is one of the Trinity paths
pub fn validate_trinity_path(path: &Path) -> bool {
    path == boru_socket_path() || path == yomi_socket_path() || path == suji_socket_path()
}
