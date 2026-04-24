# Changelog

All notable changes to BORU are documented here.

Format: [Semantic Versioning](https://semver.org/) — `MAJOR.MINOR.PATCH`

---

## [0.3.0] — 2026-04-24

### Added
- **Packaging infrastructure** for Linux and macOS distribution
  - Debian/Ubuntu `.deb` packaging (control, postinst, postrm scripts)
  - Fedora/RHEL `.rpm` spec file
  - macOS Homebrew formula
  - Universal `install.sh` script with auto-detection for all supported distros
  - `build-packages.sh` script for creating distribution packages
- **Platform support matrix** in README
  - Debian / Ubuntu / Linux Mint / Pop!_OS
  - Fedora / RHEL / Rocky / AlmaLinux
  - Arch / Manjaro / EndeavourOS
  - openSUSE
  - macOS (Intel & Apple Silicon)
- Comprehensive distribution-specific install instructions in README
- Post-install runtime directory setup (socket, quarantine, audit, hash DB)

### Changed
- README rewritten with full install guides per distribution
- Project scoped exclusively for Linux and macOS (Unix socket requirement)

---

## [Unreleased]

### Added
- Initial repository scaffold
- Trinity socket contract stubs (`/tmp/momo/boru.sock`, `/tmp/momo/zuno.sock`, `/tmp/momo/saba.sock`)
- Protocol gates (AGENTS.md) — 7 gates defined
- CLAUDE.md agent instructions
- Architecture documentation
- WASM sandbox core (`src/cage/`)
- Syscall intercept layer (`src/intercept/`)
- Audit logging with tamper-proof SHA-256 chain (`src/intercept/audit.rs`)
- Quarantine system (`src/intercept/quarantine.rs`)
- Unix socket server (`src/socket/`)
- Ratatui TUI dashboard (`src/tui/`)
- Directory scanner (`src/scanner/`)
- File watchdog with real-time monitoring (`src/watchdog/`)
- Hash database for threat detection (`src/threat/`)
- Session replay (`src/session/`)
- Shadow rollback (`src/shadow/`)
- Agent IAM system (`src/iam/`)
- Runner/dependency management (`src/runner/`)
- CLI commands: cage, check, daemon, tui, log, quarantine, deps, scan, watch, hash, replay, rollback, iam

---

## [0.1.0] — TBD (Initial Release)

### Planned
- First stable public release
- CI/CD pipeline for automated package builds
- Man page (`boru.1`)
- Shell completion scripts (bash, zsh, fish)