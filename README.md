# ChatGPT Island

Windows Tauri 2 / Rust background utility using the actual ChatGPT website in a private WebView2 popup. No API key, custom chat interface, backend, OAuth or application account is required. Guest availability depends on ChatGPT and the current network/location.

Milestone 01 guest feasibility is verified. Milestone 02 background/tray/shortcut implementation is in place; see [native verification results](docs/MILESTONE02.md) for passed and outstanding checks. Milestones 03–05 have not been implemented.

## Use

Start with `npm run tauri dev` during development. Normal startup creates a tray icon and **no launcher or popup**.

- **Alt + Shift + E** opens a fresh private popup; pressing it again requests closure.
- **Escape** requests closure only inside the focused popup. It is handled natively by WebView2, not by website scripts or a global Escape shortcut.
- Tray **Open ChatGPT Island** opens or focuses the current popup. Repeated opens do not replace a conversation.
- Tray **Settings** shows shortcut status and the close-warning preference. **Show close warning again** resets that preference.
- Tray **Quit** requests application exit, confirming conversation loss when a popup is open and warnings are enabled.
- A normal left-click on the tray icon opens/focuses the popup; right-click opens its menu. Windows may place the icon under hidden icons.

The trusted close dialog offers Cancel, Close conversation and Don't show this again. Cancel preserves the popup and never saves the checkbox. Confirm destroys the WebView. Selecting the checkbox and confirming persists the skip-warning preference. Without it, every close request shows the warning again. The native X button follows the same rules. Closing a popup leaves the tray app running.

Escape takes precedence over ChatGPT's own menus/dialogs while its WebView has focus. Cancel the local confirmation to keep the conversation. Escape elsewhere has no island effect. Forceful process termination, crashes and OS shutdown cannot be guaranteed to display confirmation.

## Development requirements

Windows 10/11 x64, Node.js 24 LTS/npm, stable Rust MSVC, Microsoft C++ Build Tools, Windows SDK and WebView2 Evergreen Runtime. The verified environment has Rust/Cargo 1.99.1, Build Tools 2022 17.14.44, SDK 10.0.26100.0 and WebView2 157.0.4341.42. See [DEVELOPMENT.md](docs/DEVELOPMENT.md) for exact official installation instructions.

Docker is the default for supported frontend checks:

```powershell
docker desktop start
docker compose build
docker compose run --rm dev
```

Native Windows checks and execution:

```powershell
npm ci
.\scripts\verify.ps1
npm run tauri dev
```

Docker installs locked JavaScript dependencies, runs TypeScript/security tests and builds frontend assets. Windows is required for Rust, tray, global shortcut and WebView2 GUI verification. Cargo.lock and package-lock.json are committed. Installer generation remains disabled.

The optional development launcher is available only in debug builds:

```powershell
npm run tauri dev -- -- --dev-launcher
```

Its close control requests confirmation, and its native X requests application Quit. It is not required during normal operation.

## Privacy, security and cleanup

Every opening uses incognito mode and a fresh UUID directory under `%LOCALAPPDATA%\com.chatgptisland.dev\private-sessions`. Native code verifies WebView2's actual private-mode flag. Closing destroys the window and invalidates its session reference. Previous directories are never reused.

Profile directories remain retained because no general browser-process-exit cleanup protocol is implemented. They may contain runtime metadata or other residual files. Exit the app and verify all processes associated with its exact private-sessions paths have ended before any offline removal. Never remove active or unrelated browser profiles. Closing does not erase remote ChatGPT data or promise forensic local erasure.

Only the boolean `skip_close_confirmation` is persisted in `%APPDATA%\com.chatgptisland.dev\settings.json`. Corrupt settings default to warnings enabled. No conversation text, selected text, cookies or tokens are stored in settings.

Remote content has no Tauri capabilities. Only narrowly scoped local launcher, confirmation and settings windows can invoke their respective controls. There are no filesystem/shell/shortcut permissions for websites or frontend code. Popup navigation is restricted to exact HTTPS chatgpt.com on port 443; new windows and downloads are denied. No authentication or challenge bypass is provided.

## Troubleshooting

- Shortcut unavailable: use the tray menu; another application may own Alt + Shift + E. Quit the conflicting app and restart Island. Do not run duplicate Island instances unintentionally.
- No launcher visible: normal behavior; use the tray icon or shortcut.
- Blank/offline page or guest restriction: inspect your connection and the website. Record login/challenge requirements rather than bypassing them.
- Popup creation failure: Settings displays safe diagnostics; check WebView2 and profile-directory permissions.
- Missing Cargo/linker: follow the Windows setup guide and reopen the terminal.
- A Chromium window-class shutdown diagnostic was observed in Milestone 01 despite successful process exit; see feasibility notes.

See [architecture](docs/ARCHITECTURE.md), [requirements](docs/REQUIREMENTS.md), [progress](docs/PROGRESS.md), [Milestone 01 evidence](docs/FEASIBILITY.md), and [Milestone 02 verification](docs/MILESTONE02.md).
