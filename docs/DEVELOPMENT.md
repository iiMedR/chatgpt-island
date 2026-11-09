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

## Milestone 02 debugging

Normal `npm run tauri dev` launches the tray utility with no local window. Press Alt + Shift + E or use the tray menu. Right-click the icon (possibly in hidden icons) for Open, Settings and Quit. The optional debug launcher is:

```powershell
npm run tauri dev -- -- -- --dev-launcher
```

The launcher is not part of normal startup. Its native X requests Quit; Request close follows the same confirmation policy as the shortcut. Closing Settings leaves the background app alive.

If the shortcut fails, Settings opens with the conflict diagnostic and the tray remains usable. Ensure only the intended Island instance is running, quit conflicting applications, and restart. Registration is performed in Rust; do not grant global-shortcut frontend permissions to debug delivery. Only the initial key press should toggle; release resets the pressed latch.

Lifecycle transitions are covered by Rust tests. For native diagnosis, use the launcher's state display in debug mode. Expected states: Closed -> Opening -> Open -> AwaitingCloseConfirmation -> Closing -> Closed. Cancel returns to Open. Repeated close requests should show only one confirmation. Creation/destruction errors appear in Settings and the development terminal, without conversation content.

Escape is handled by the popup controller's native AcceleratorKeyPressed callback, not global registration. Test once in the focused popup and once in another application. It intentionally takes precedence over the website's own Escape handling. Local confirmation Escape cancels. Never add privileged webpage IPC as a workaround.

Settings are stored in the app configuration directory, normally `%APPDATA%\com.chatgptisland.dev\settings.json`. Only skip_close_confirmation and clipboard_fallback_enabled are allowed. Both default to false; old one-field settings migrate without enabling copy. Use Settings -> Show close warning again to reset; do not modify cookies or other browser profiles. Cancel must not persist the checkbox. Confirm with the checkbox must survive app restart. Filesystem errors must preserve the live conversation.

For shutdown testing, Tray Quit with a popup should confirm when warnings are enabled; Cancel must keep both app and conversation. Confirm must destroy the popup and exit. Verify no app-owned WebView2 processes remain, then verify a subsequent process can register the shortcut. Do not treat OS force termination as proof of orderly unregistration or tray Quit behavior.

The automation tool in this session can target native application windows but does not expose the taskbar. Tests of physical tray menu interaction therefore require a user action if no targetable shell surface is available. Record those tests accurately instead of inferring success from TrayIconBuilder compilation.


## Milestone 03 native capture diagnostics

Use non-sensitive fixtures only. Docker checks TypeScript, capability boundaries and builds; Windows checks COM/UIA, clipboard and shortcuts. Do not infer application compatibility from unit tests. See TEXT_CAPTURE_COMPATIBILITY.md for observations and remaining tests.

For an explicit sample preview, ensure only one Island process owns the shortcut, run Vite (`npm run dev`) in one terminal, then in another:

```powershell
cd src-tauri
cargo run --locked -- --dev-launcher --dev-selection-preview
```

The preview requires both a debug build and the explicit flag. Select sample text in another application, press Alt + Shift + E, return to the launcher and click Refresh sample selection preview. The status reports method, character count and elapsed capture milliseconds, without text. The website never receives the selection. Close the popup, verify Closed, then verify no pending selection on another preview request. Avoid saving preview screenshots containing personal text.

UIA failures may mean unsupported selection patterns, an inaccessible control, integrity-level restrictions, a changed foreground window, or a provider timeout. Island does not elevate or enable accessibility modes automatically. Keep the source foreground until capture completes. A visible selection does not guarantee provider support (see the VS Code result). There is no continuous monitoring or clipboard polling outside an explicit capture attempt.

To test copy fallback, enable it yourself in the trusted Settings window after reading its explanation; it is a privacy preference. It only operates if UIA proves a selection but GetText fails, and the clipboard is empty. Do not clear personal clipboard data just to force a test. Use an isolated non-sensitive test session and a known compatible provider. Existing text/rich text/image/file formats must cause refusal. For concurrent updates, verify changed sequence/owner prevents restoration. For delayed rendering or failed restoration, expect a trusted warning and inspect the clipboard yourself. Mock tests cover these decisions; they do not establish native clipboard-manager compatibility. Disable the preference when finished.

The settings migration accepts the prior one-boolean file and never enables fallback implicitly. Selected text is not part of the serialized settings schema. Native test execution may need the normal Windows process context because the Codex filesystem sandbox can deny MoveFileExW on test temporary files; this is a test-environment restriction, not permission to weaken atomic preference writes.

## Milestone 04 handoff and window diagnostics

Keep verification focused: run the existing Docker/native pass once, then one non-sensitive selection → draft workflow. Repeat only a failed check affected by a fix. Do not submit sample messages automatically. Unit/fixture tests need neither live ChatGPT nor paid requests. CHATGPT_HANDOFF.md separates observed results from untested platform cases.

The native adapter source is `src/composer-script.js`, embedded by `handoff/mod.rs`; frontend hot reload does not update the embedded adapter. Rebuild the Rust app after editing it. Safe returned statuses are waiting/ready/inserted/accepted/occupied/blocked/changed/unavailable. Selected text and full DOM must never be added to diagnostics. The existing debug selection preview remains explicitly gated and should be used only for samples.

If the composer is unavailable, inspect the real page visually: login/challenge/redirect must stop the handoff, never trigger a bypass. Verify a unique visible editable control on the exact root ChatGPT document before adapting selectors. At 520px the observed site uses `mobile-composer-prompt`; the desktop adapter also knows `prompt-textarea`. Do not widen matching to arbitrary textareas, scrape conversations, or repeatedly overwrite a draft to force recognition. A native value setter/input event serves textarea controls; contenteditable uses compatibility-sensitive browser editing. An enabled Send plus preserved draft after additional typing is stronger evidence than DOM text alone. Never activate Send in an automated smoke test.

The trusted failed-handoff notice provides explicit user selection/copy rather than automatic clipboard writes. If a callback is uncertain, check the composer before pasting; automatic insertion is never retried. The notice and pending state are session-owned and must disappear with confirmed destruction. Cancel preserves the page. Slow loading beyond the 15-second readiness bound can deliberately fall back.

For borderless geometry, `window_layout.rs` uses source HWND/cursor monitor working area and physical coordinates scaled from logical targets. Do not clamp negative coordinates to zero. Display bounds exclude taskbars. Initial hidden creation prevents a visible primary-monitor flash. Native movement/resizing/scaling events re-check visibility. Alt+F7/Alt+F8 invoke Windows system move/size while the popup is focused; arrow keys and Enter complete the operation. No site drag IPC or mouse overlay exists. DWM corners are best-effort on Windows 11; Windows 10 and mixed-DPI/hot-plug cases need separate environments, not claims inferred from unit tests.

## Production packaging

Use `npm run release:build` for the supported x64 NSIS route, rather than manually copying a developer build. It invokes version/resource checks, a release build, dependency notice generation, compiler path remapping, stable asset naming, checksums and package auditing. Missing supplemental licenses fail the build; keep their source/checksum records and LF byte stability intact. The app identifier is unchanged so existing settings paths remain compatible.

See RELEASING.md for the tag-only draft workflow, manual-publication gates and website download contract. See INSTALLATION.md for end-user instructions and RELEASE_BUILD_REPORT.md for actual local output evidence. Existing milestone observations remain historical; a build is not a substitute for clean-machine installer acceptance.
