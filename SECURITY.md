# Security policy

## Support and project status

The application is pre-1.0. Only the latest published release is intended to receive security fixes; historical development builds are unsupported. No release or clean-machine installation acceptance is implied by this policy. Read [release readiness](docs/RELEASING.md) before distributing binaries.

ChatGPT Island is independent and is not affiliated with, endorsed by, or sponsored by OpenAI. OpenAI website/service security concerns should be reported to that service; this policy covers the native application and distribution pipeline.

## Report a vulnerability

Use GitHub's **Report a vulnerability** action at https://github.com/iiMedR/chatgpt-island/security/advisories/new **if private vulnerability reporting has been enabled by the maintainer**. Enabling it is a publishing prerequisite; it has not been enabled by this work. No private email address or response-time SLA is currently advertised. If private reporting is unavailable, request that a private channel be enabled without posting exploit details, personal selections, tokens or browser profiles in a public issue.

Include the affected version/commit, Windows and WebView2 versions, reproduction steps with synthetic data, expected/actual behavior, impact, and a minimal proof of concept. Remove personal data. Allow maintainers time to investigate before public disclosure; no guaranteed response deadline is claimed.

## Security properties

Remote ChatGPT has no privileged Tauri capability. Local controls are separated by exact window labels and command authorization. UIA capture is explicit and bounded; clipboard fallback is off by default. Drafts are not submitted automatically. Selection and handoff state are memory-only from the application's perspective. Browser profiles are fresh and private mode is verified, but residual directories are retained until safe offline cleanup.

Preparing text in the real ChatGPT page makes it visible to that page's scripts before Send. The app cannot guarantee website-side draft processing, remote retention or forensic local erasure. Protected controls and clipboard managers also impose documented limits.

## Distribution

Download only from this repository's GitHub Releases. Compare the installer SHA-256 against the same release's `SHA256SUMS.txt`. A checksum checks file integrity; it does not independently authenticate a compromised release account. Current builds are unsigned: SmartScreen/unknown-publisher warnings may occur. Signing certificates and updater keys must never enter Git history. No auto-updater is configured.

Tag-triggered workflows build with read-only repository access. Only the separate draft-release job has release write permission. Actions are pinned to commit SHAs. Protect release tags and configure the `release` environment with reviewers. Do not publish a draft until installation, upgrade, uninstall, functional acceptance and package review have passed.
