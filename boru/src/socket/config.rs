//! BORU Socket Configuration
//!
//! GATE 3: All socket paths centralized here. No hardcoded paths elsewhere.
//!
//! Socket paths for Project MOMO Ecosystem:
//! - BORU: [TEMP]/momo/boru.sock (security engine)
//! - NUKI: [TEMP]/momo/nuki.sock (search/memory engine)
//! - SUJI: [TEMP]/momo/suji.sock (orchestrator/conductor)
//! - ZUNO: [TEMP]/momo/zuno.sock (indexer - Phase 2 stub)
//! - SABA: [TEMP]/momo/saba.sock (router - Phase 2 stub)
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

/// NUKI socket path (search engine - auto-detected)
pub fn nuki_socket_path() -> PathBuf {
    momo_base_dir().join("nuki.sock")
}

/// SUJI socket path (orchestrator - auto-detected)
pub fn suji_socket_path() -> PathBuf {
    momo_base_dir().join("suji.sock")
}

/// ZUNO socket path (Phase 2 - stub only)
#[allow(dead_code)]
pub fn zuno_socket_path() -> PathBuf {
    momo_base_dir().join("zuno.sock")
}

/// SABA socket path (Phase 2 - stub only)
#[allow(dead_code)]
pub fn saba_socket_path() -> PathBuf {
    momo_base_dir().join("saba.sock")
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

/// Check if NUKI (search engine) is available
pub fn nuki_available() -> bool {
    is_service_available(nuki_socket_path())
}

/// Check if SUJI (orchestrator) is available
pub fn suji_available() -> bool {
    is_service_available(suji_socket_path())
}

/// Get ecosystem status - which siblings are present
pub fn ecosystem_status() -> EcosystemStatus {
    EcosystemStatus {
        boru: true, // We're here
        nuki: nuki_available(),
        suji: suji_available(),
        zuno: is_service_available(zuno_socket_path()),
        saba: is_service_available(saba_socket_path()),
    }
}

/// Ecosystem presence detection
#[derive(Debug, Clone)]
pub struct EcosystemStatus {
    pub boru: bool,
    pub nuki: bool,
    pub suji: bool,
    pub zuno: bool,
    pub saba: bool,
}

impl EcosystemStatus {
    /// Check if we're running in full MOMO ecosystem mode
    pub fn full_ecosystem() -> bool {
        let status = ecosystem_status();
        status.boru && status.nuki && status.suji
    }

    /// Get count of available services
    pub fn service_count(&self) -> usize {
        let mut count = 0;
        if self.boru { count += 1; }
        if self.nuki { count += 1; }
        if self.suji { count += 1; }
        if self.zuno { count += 1; }
        if self.saba { count += 1; }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_paths() {
        assert_eq!(boru_socket_path(), std::env::temp_dir().join("momo/boru.sock"));
        assert_eq!(nuki_socket_path(), std::env::temp_dir().join("momo/nuki.sock"));
        assert_eq!(suji_socket_path(), std::env::temp_dir().join("momo/suji.sock"));
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
