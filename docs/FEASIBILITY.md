# Feasibility — 2026-11-09

Verdict: PARTIAL implementation, blocked from native verification by missing development prerequisites. No conclusion about ChatGPT guest support can be drawn.

## Environment evidence

| Item | Observation |
|---|---|
| OS | Windows NT 10.0.26200 x64, build 26200.9711, 25H2; registry ProductName says Windows 10 Pro (legacy label) |
| Rust / Cargo / rustup | Not installed or on PATH; user .cargo/bin/rustc.exe absent |
| MSVC / Windows SDK | Tauri info detects no suitable Visual Studio/Build Tools; standard installer and SDK include paths absent |
| Node / npm | 24.22.1 / 11.19.1 |
| WebView2 Runtime | Tauri info detects 157.0.4341.42 |
| Rust Tauri | Requested exact 2.12.5; not resolved/compiled |
| JS Tauri API / CLI | 2.12.4 / 2.12.4 |
| Wry / Tao | NOT TESTED; Cargo unavailable, no Cargo.lock |
| Generator | Official create-tauri-app 4.7.5, vanilla-ts, npm |

## Results

| Test | Result | Evidence |
|---|---|---|
| ChatGPT website loads | NOT TESTED | Native application could not launch |
| Guest mode works | NOT TESTED | No actual WebView2 guest test |
| Manual question submission | NOT TESTED | No submission made |
| Follow-up conversation | NOT TESTED | No conversation made |
| Popup destruction | NOT TESTED | destroy() implementation has not run |
| Fresh session on reopening | NOT TESTED | Unique-directory design not empirically validated |
| Private mode active | NOT TESTED | incognito is configured; runtime behavior unobserved |
| Existing browser cookies isolated | NOT TESTED | Must compare with signed-in browser |
| Launcher asset build | PASS | npm run build: TypeScript and Vite successful |
| Static security/configuration tests | PASS | npm test: 2 passing |
| Rust compilation and unit tests | NOT TESTED | cargo executable missing |

## Executed verification

- rustc --version, cargo --version, rustup show: command not recognized.
- node --version / npm --version: successful, versions above.
- npm create tauri-app@latest chatgpt-island -- --template vanilla-ts --manager npm --identifier com.chatgptisland.dev --yes: succeeded using official tooling. Initial sandbox network attempt stalled; network-enabled retry succeeded.
- npm install: succeeded; initial install 20 packages, zero reported vulnerabilities.
- npm install --package-lock-only after removing unused opener: succeeded; 20 audited packages, zero reported vulnerabilities.
- npm run build: initial CSS module type error fixed with vite/client declaration; final build PASS (7 transformed modules).
- npm test: PASS, 2 static configuration tests. These do not validate native ACL enforcement or guest behavior.
- npm run tauri info: detects WebView2 and missing Rust/MSVC/SDK prerequisites.
- npm run tauri dev: FAIL to start: `cargo metadata --no-deps --format-version 1: program not found`.
- cargo check, cargo test, cargo fmt --check, cargo clippy from src-tauri: could not execute; cargo command not recognized. Rust tests are written, not claimed passed.

## Required manual Windows procedure

1. Install documented prerequisites. Run all verification commands; resolve any compiler, formatting, lint, or ACL-generation failures. Preserve Cargo.lock and record the actual Rust, Tauri and Wry versions.
2. Run npm run tauri dev. Confirm launcher and runtime initialize without errors. In a separate existing browser, confirm its signed-in state without copying any cookies or credentials.
3. Open the private popup. Verify rendering, native ChatGPT composer and no inherited account. Record blank pages, runtime errors, login requirements, browser incompatibility, verification challenges or regional restrictions. Stop if access requires authentication or bypassing a challenge.
4. If guest mode is available, manually ask: `Remember the test phrase island violet telescope 731. What is 2 plus 2?` Record whether submission and response work. Do not automate service queries.
5. Manually ask `What test phrase did I give you?` and verify within-window context. Use only non-sensitive content.
6. Destroy using the launcher. Confirm native window disappears, terminal reports destruction, status returns idle, and runtime no longer has the popup window. Also test the popup's native X button.
7. Reopen. Check the old conversation is absent and a fresh guest interaction starts. Ask about the phrase without supplying it; distinguish guesses from restored state. Repeat at least three times and after application restart. Record each run, actual profile paths, account state, and observations. Do not infer success from a new identifier alone.
8. Verify private mode through WebView2 diagnostics/profile inspection in a local debug session; record IsInPrivateModeEnabled and separate environment user-data folders. Match observations to the resolved Wry version, especially older-runtime fallback behavior.
9. Attempt duplicate opens, open during close, close while idle, shutdown with popup active, offline startup and reconnect/reopen. Failures must leave controls usable without crashing.
10. Test unrelated links/new-window/download requests are blocked and remote content has no Tauri capability access; never collect tokens or credentials.

## Limitations

No native test was possible. Top-level navigation is restricted to exactly https://chatgpt.com; other-origin login and redirects may be blocked. Website error pages provide network diagnostics; the app does not programmatically classify guest restrictions or navigation errors. Incognito environments may share private state when reused, which motivates unique directories. Directories are not automatically removed: native window destruction does not establish WebView2 browser-process exit. Residual files are possible, and remote retention is independent of local lifecycle. Do not mark Milestone 01 complete until all required runtime tests pass.
