//! BORU Socket Configuration
//!
//! GATE 3: All socket paths centralized here. No hardcoded paths elsewhere.
//!
//! Socket paths for Project MOMO Trinity:
//! - BORU: /tmp/momo/boru.sock (this engine)
//! - ZUNO: /tmp/momo/zuno.sock (indexer - stub only)
//! - SABA: /tmp/momo/saba.sock (router - stub only)

/// Default BORU socket path
pub const BORU_SOCKET_PATH: &str = "/tmp/momo/boru.sock";

/// ZUNO socket path (Phase 2 - stub only)
#[allow(dead_code)]
pub const ZUNO_SOCKET_PATH: &str = "/tmp/momo/zuno.sock";

/// SABA socket path (Phase 2 - stub only)
#[allow(dead_code)]
pub const SABA_SOCKET_PATH: &str = "/tmp/momo/saba.sock";

/// Maximum request size: 10MB
pub const MAX_REQUEST_SIZE: usize = 10 * 1024 * 1024;

/// Socket directory path
#[allow(dead_code)]
pub const SOCKET_DIR: &str = "/tmp/momo";

/// BORU workspace directory for sandbox file operations
#[allow(dead_code)]
pub const BORU_WORKSPACE_DIR: &str = "/tmp/momo/workspace";
