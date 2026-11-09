# Changelog

All notable user-facing changes are documented here. Version tags use `vMAJOR.MINOR.PATCH` and must match package.json, Cargo.toml and tauri.conf.json.

## [Unreleased]

### Added

- Windows x64 per-user NSIS packaging with WebView2 bootstrapper handling.
- Supplied application branding converted to Tauri/Windows icons.
- Tag-only Windows verification/build pipeline and separate draft-release job.
- Stable installer/checksum filenames and release metadata for website distribution.
- MIT project license, contribution/security policies and dependency notices.
- Release preflight checks and explicit production resource allowlist.

### Changed

- Development launcher code is excluded from the production frontend bundle.
- Installer bundling is enabled; the existing application identifier is preserved for settings compatibility.

### Release status

These changes are local release infrastructure, not an announcement of a published 0.1.0 release. Clean-machine install/upgrade/uninstall and outstanding runtime acceptance remain required. No binaries have been uploaded by this work.

## 0.1.0 — development baseline (not a published release)

- Tray-based private ChatGPT WebView2 popup and native keyboard controls.
- Explicit UI Automation selection capture and optional clipboard fallback.
- Session-bound editable draft handoff, borderless window and monitor-aware layout.
- Manual handoff available only from Settings; no automatic message submission.
