# Development

## Two supported workflows

Docker is the default for frontend dependency installation, TypeScript checks, JavaScript tests and frontend asset builds. Windows is required for Rust MSVC compilation, native tests, the Tauri GUI and Microsoft WebView2. Linux Docker results do not establish Windows GUI or ChatGPT compatibility.

## Windows host setup

Use Node.js 24 LTS, npm, Git, stable Rust MSVC, Microsoft C++ Build Tools and Windows SDK. WebView2 Evergreen Runtime is required at runtime.

Install Rust from the official Windows installer at https://rustup.rs/:

```powershell
.\rustup-init.exe -y --default-host x86_64-pc-windows-msvc --default-toolchain stable --profile default
```

This installs Cargo, rustfmt and Clippy. Reopen your terminal after installation. For an existing PowerShell session, update only that session:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
rustc --version
cargo --version
rustup show
```

Download Build Tools from https://visualstudio.microsoft.com/visual-cpp-build-tools/. Approve Windows elevation if requested. The official 2022 bootstrapper accepts:

```powershell
.\vs_BuildTools.exe --passive --wait --norestart --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended
```

Wait for installation to finish. Do not treat launching the bootstrapper as installation success. Check `vswhere` and `npm run tauri info`; follow any restart instruction. If Cargo still cannot find the linker, use a fresh terminal or Developer PowerShell for Visual Studio. Do not replace the MSVC target with GNU to bypass missing prerequisites.

## Docker setup and default checks

Use the existing Docker Desktop installation if present. Otherwise install the official distribution from https://www.docker.com/products/docker-desktop/ and follow https://docs.docker.com/desktop/setup/install/windows-install/ for WSL 2 requirements. Approve required system changes yourself. This project does not configure Windows containers.

```powershell
docker desktop start
docker --version
docker compose version
docker info
docker compose build
docker compose run --rm dev
```

The image uses a digest-pinned Node 24 Debian slim base and `npm ci` against package-lock.json. It runs as UID 1000 (`node`), drops Linux capabilities, and exposes no ports, host mounts or Docker socket. `.dockerignore` allowlists only build inputs; installers, Git data, environment files, profiles and native build artifacts are excluded. Node image updates require explicitly updating the digest and retesting.

The default command runs `scripts/verify-docker.sh`, stopping on the first failed step. Individual commands:

```powershell
docker compose run --rm dev npm ci --no-audit --no-fund
docker compose run --rm dev npm run typecheck
docker compose run --rm dev npm test
docker compose run --rm dev npm run build
```

Rebuild after editing source or lockfiles: files are copied into the image rather than bind-mounted. Generated dist assets stay in the disposable container. No database or API backend is needed. The image deliberately does not install Linux Tauri GUI dependencies or pretend to test WebView2.

## Native verification and application launch

From the project root:

```powershell
npm ci
.\scripts\verify.ps1
npm run tauri dev
```

The PowerShell script resolves the project path from its own location, preserves the caller's working directory, stops on failures and prints the failing step. It runs TypeScript, JavaScript tests, frontend build, then `cargo check --locked`, `cargo test --locked`, `cargo fmt --check` and `cargo clippy --locked -- -D warnings`. No administrator rights are needed for verification. Cargo.lock is committed for reproducible resolution.

Use Open private popup and Destroy popup. Wait for idle before reopening. Closing the launcher exits the app. Follow docs/FEASIBILITY.md for actual website, guest-conversation, destruction and isolation tests. Do not log in, bypass challenges or use undocumented APIs to force guest feasibility.

## WebView2 diagnostics and privacy

`npm run tauri info` detects the runtime version. Native creation errors appear in the launcher and development terminal. Creation success is not network success; inspect the page for offline, login or verification errors. Do not enable credential or token capture. No website scripts are injected for diagnostics.

Each popup uses incognito plus a unique application-owned profile directory. Safe automatic cleanup remains unimplemented: destroying a window does not prove every WebView2 process has exited. Never delete active profile files. To clean stale directories, first exit this app and confirm all WebView2 processes associated with its exact private-sessions directory have ended, then remove only that directory. Do not delete another browser's data or terminate unrelated WebView2 processes. If ownership or release is uncertain, retain the files. Local cleanup does not erase remote ChatGPT data.

## Common errors

- Docker named pipe missing: start Docker Desktop and wait for `docker info` to return server information.
- WSL or virtualization failure: follow Docker's official host setup; system approval or reboot may be required.
- `link.exe not found`: C++ installation is missing, incomplete or not detected; confirm MSVC workload and SDK.
- `cargo` not found: reopen the terminal or apply the process-local PATH refresh above.
- Container shell reports `build\r`: ensure shell scripts use LF; `.gitattributes` enforces this for Git checkouts.
- Guest login/challenge: record the restriction and stop; it is not evidence of an application compiler failure.
