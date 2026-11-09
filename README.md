# ChatGPT Island

Milestone 01 Windows-only Tauri 2 / Rust feasibility prototype. A minimal Vanilla TypeScript development launcher opens the actual ChatGPT website; there is no custom chat UI, API key, backend, OAuth, account integration, shortcut, tray, or selection capture.

**Milestone 01: COMPLETE on the tested Windows host/network. Native guest question, response, follow-up and fresh-session reopening/restart passed. Profile directories remain retained; automatic deletion is not implemented. See docs/FEASIBILITY.md for evidence and untested reliability cases.**

## Prerequisites

Inspected on 2026-11-09: Windows build 26200.9711, Node 24.22.1, npm 11.19.1, WebView2 157.0.4341.42. The follow-up setup installed Rust 1.99.1 (stable MSVC), Visual Studio Build Tools 2022 17.14.44 and Windows SDK 10.0.26100.0. Docker Desktop was already installed and its Linux engine is now operational.

1. Download [Visual Studio Build Tools 2022](https://aka.ms/vs/17/release/vs_BuildTools.exe). In the installer select **Desktop development with C++**, MSVC x64/x86 build tools, and a Windows 10/11 SDK. Complete installation and restart your terminal.
2. Download and run the Windows installer from [rustup](https://rustup.rs/). Choose stable and the `x86_64-pc-windows-msvc` host. Then run:

   ```powershell
   rustup default stable-msvc
   rustup component add rustfmt clippy
   rustc --version
   cargo --version
   rustup show
   ```
3. If missing, install the Evergreen WebView2 Runtime from [Microsoft](https://developer.microsoft.com/en-us/microsoft-edge/webview2/). It is already detected here.
4. Use Node.js LTS and npm. See [official Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

## Development

From this project directory:

```powershell
npm ci
npm test
npm run build
npm run tauri info
npm run tauri dev
```

Use **Open private popup**, interact directly with ChatGPT, then use its native close button or **Destroy popup**. Wait for `idle` before opening a fresh popup. Closing the launcher exits the application. These are development controls; Escape and global shortcuts are future work.

Rust verification, from `src-tauri`:

```powershell
cargo check
cargo test
cargo fmt --check
cargo clippy -- -D warnings
```

`npm run build` builds only the launcher assets. It does not compile Rust. Installer bundling is disabled. Cargo.lock is committed. Rust Tauri is pinned to 2.12.5; Wry resolves to 0.57.1. Run scripts/verify.ps1 for the complete native check sequence.

## Privacy and limitations

Each opening requests incognito mode and a newly created UUID-named data directory under `%LOCALAPPDATA%\com.chatgptisland.dev\private-sessions`. Previous directories are never reused. The app additionally checks the actual WebView2 profile private-mode flag and rejects unverified sessions. The native UI tests observed fresh conversation state across reopening and application restart. Browser profiles can share state when using the same environment, so incognito alone is insufficient.

The app destroys windows rather than hiding them. It never stores or logs conversation text. Profile directories may contain runtime metadata or other residual data; they are deliberately retained because window destruction does not prove browser-process exit. No automatic deletion is attempted. To remove them, exit the app and confirm no WebView2 processes associated with these directories remain, then delete only this application's private-sessions directory. Do not delete another browser's profiles. Closing does not erase ChatGPT server data or guarantee forensic erasure.

Only HTTPS navigation to exactly chatgpt.com on port 443 is allowed. New windows and downloads are denied. Other-origin redirects, including login, are blocked; this may affect guest feasibility. Network subresources remain controlled by the website, not by this top-level navigation policy. No challenges or restrictions are bypassed.

## Troubleshooting

- `cargo metadata ... program not found`: install Rust and reopen the terminal.
- Missing linker or SDK: repair Build Tools with the workload above.
- WebView creation failure: check WebView2 installation, profile directory permissions, and terminal diagnostics.
- Blank/offline/error page: inspect your network, destroy and reopen; popup creation is not a successful-load signal.
- Login/challenge/region restriction: record the observation and stop; do not bypass it.
- Already active/closing: wait for idle or close the existing popup. Destruction failure permits a retry.

## Docker default workflow

```powershell
docker desktop start
docker compose build
docker compose run --rm dev
```

Docker installs locked JavaScript dependencies and runs TypeScript, static security tests and the frontend build. It runs as a non-root user with no host mounts or published ports. Windows remains required for native Rust builds and all WebView2 GUI testing. See [development guide](docs/DEVELOPMENT.md) for setup, individual commands and debugging.

See [feasibility procedure and results](docs/FEASIBILITY.md), [architecture](docs/ARCHITECTURE.md), [requirements](docs/REQUIREMENTS.md), and [progress](docs/PROGRESS.md).
