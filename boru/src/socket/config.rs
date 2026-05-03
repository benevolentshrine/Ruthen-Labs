//! BORU Socket Configuration
//!
//! GATE 3: All socket paths centralized here. No hardcoded paths elsewhere.
//!
//! Socket paths for Project MOMO Ecosystem:
//! - BORU: [TEMP]/momo/boru.sock (security engine)
//! - NUKI: [TEMP]/momo/nuki.sock (search/memory engine)
//! - SUJI: [TEMP]/momo/suji.sock (orchestrator/conductor)
//! - YOMI: [TEMP]/momo/yomi.sock (indexer - Phase 2 stub)
//! - SUJI: [TEMP]/momo/suji.sock (router - Phase 2 stub)
//!
//! Unix Philosophy: Auto-discover siblings via filesystem sockets.
//! No hardcoded ports. No localhost HTTP. Pure Unix sockets.

use std::path::{Path, PathBuf};

/// Base directory for MOMO ecosystem
pub fn momo_base_dir() -> PathBuf {
    std::env::temp_dir().join("momo")
}

/// Default BORU socket path
pub fn boru_socket_path() -> PathBuf {
    momo_base_dir().join("boru.sock")
}

/// SUJI socket path (orchestrator)
pub fn suji_socket_path() -> PathBuf {
    momo_base_dir().join("suji.sock")
}

/// YOMI socket path (indexer)
#[allow(dead_code)]
pub fn yomi_socket_path() -> PathBuf {
    momo_base_dir().join("yomi.sock")
}

/// Maximum request size: 10MB
pub const MAX_REQUEST_SIZE: usize = 10 * 1024 * 1024;

/// Socket directory path
pub fn socket_dir() -> PathBuf {
    momo_base_dir()
}

/// BORU workspace directory for sandbox file operations
#[allow(dead_code)]
pub fn boru_workspace_dir() -> PathBuf {
    momo_base_dir().join("workspace")
}

/// Service discovery: Check if a sibling service is running
pub fn is_service_available<P: AsRef<Path>>(socket_path: P) -> bool {
    socket_path.as_ref().exists()
}

/// Check if SUJI (orchestrator) is available
pub fn suji_available() -> bool {
    is_service_available(suji_socket_path())
}

/// Check if YOMI (indexer) is available
pub fn yomi_available() -> bool {
    is_service_available(yomi_socket_path())
}

/// Get ecosystem status - which siblings are present
pub fn ecosystem_status() -> EcosystemStatus {
    EcosystemStatus {
        boru: true, // We're here
        suji: suji_available(),
        yomi: yomi_available(),
    }
}

/// Ecosystem presence detection
#[derive(Debug, Clone)]
pub struct EcosystemStatus {
    pub boru: bool,
    pub suji: bool,
    pub yomi: bool,
}

impl EcosystemStatus {
    /// Check if we're running in full MOMO ecosystem mode
    pub fn full_ecosystem() -> bool {
        let status = ecosystem_status();
        status.boru && status.suji && status.yomi
    }

    /// Get count of available services
    pub fn service_count(&self) -> usize {
        let mut count = 0;
        if self.boru { count += 1; }
        if self.suji { count += 1; }
        if self.yomi { count += 1; }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_paths() {
        assert_eq!(boru_socket_path(), std::env::temp_dir().join("momo/boru.sock"));
        assert_eq!(suji_socket_path(), std::env::temp_dir().join("momo/suji.sock"));
        assert_eq!(yomi_socket_path(), std::env::temp_dir().join("momo/yomi.sock"));
        assert_eq!(MAX_REQUEST_SIZE, 10 * 1024 * 1024);
    }

    #[test]
    fn test_ecosystem_status() {
        // Just verify it doesn't panic
        let status = ecosystem_status();
        assert!(status.boru); // We exist
        // Other values depend on runtime state
    }
}
