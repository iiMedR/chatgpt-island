# Contributing to ChatGPT Island

ChatGPT Island is an independent Windows utility, not affiliated with, endorsed by, or sponsored by OpenAI. Contributions to this repository are offered under the [MIT license](LICENSE); third-party dependencies retain their licenses.

## Setup

Use Windows x64, Node 24, Rust MSVC, Microsoft C++ Build Tools/Windows SDK and WebView2. Start with `npm ci` and `npm run tauri dev`. See [development](docs/DEVELOPMENT.md). CI pins Node 24.22.1 and Rust 1.99.1; lockfiles are committed. Docker covers the frontend only.

Discuss architectural changes or large features in an issue first. Keep PRs focused, use conventional commit messages, explain the user-visible change and cite the verification actually performed. Preserve existing work and avoid unrelated formatting changes.

## Verification

Run the checks appropriate to a change, once. The full Windows entry point is `.\scripts\verify.ps1`; the Docker entry point is `docker compose build` then `docker compose run --rm dev`. Packaging uses `.\scripts\build-release.ps1`. Do not claim native compatibility from mock tests alone.

## Boundaries to preserve

- Remote ChatGPT content receives no native Tauri permissions or host objects.
- Capture only explicit selections after the approved shortcut, before popup focus.
- Never log selected text, drafts, tokens, cookies or private profile data.
- Never submit a message, bypass login/challenges, or call undocumented ChatGPT APIs.
- Keep bounded capture, session ownership, cancellation and exactly-once insertion.
- Manual handoff windows open only on user request from Settings.
- Do not delete active browser profiles or silently overwrite clipboard formats.
- Never commit API keys, signing certificates, `.env` files, binary artifacts or personal test data.

Use synthetic non-sensitive fixtures in screenshots/reports. Report vulnerabilities privately as described in [SECURITY.md](SECURITY.md). Only maintainers may authorize version tags and releases; ordinary changes must not publish binaries.
