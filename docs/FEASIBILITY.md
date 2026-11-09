# Feasibility — 2026-11-09

**Milestone 01 verdict: COMPLETE for the tested Windows host and network.** Actual guest questions, a response, within-session context, popup destruction, repeated fresh openings and application restart were verified through the native UI. This is not a guarantee of guest availability in other regions or future website versions. Automatic profile-directory deletion remains unimplemented; retention is the documented safe fallback permitted by this milestone's cleanup investigation.

## Verified environment

| Component | Actual observation |
|---|---|
| Windows | NT 10.0.26200 x64, build 26200.9711, 25H2 |
| Rust | rustc 1.99.1 (58cf8f9c3 2026-10-13), stable-x86_64-pc-windows-msvc |
| Cargo / rustup | 1.99.1 / 1.29.3 |
| Build Tools | Visual Studio Build Tools 2022 17.14.44, installation 17.14.37827.9; vswhere isComplete=1 |
| MSVC / SDK | 14.44.35207 / Windows SDK 10.0.26100.0 |
| WebView2 | 157.0.4341.42, detected by Tauri; not reinstalled |
| Native dependencies | Tauri 2.12.5, tauri-build 2.7.4, Wry 0.57.1, Tao 0.37.2; committed Cargo.lock |
| JavaScript tools | Host Node 24.22.1, npm 11.19.1, Tauri API/CLI 2.12.4 |
| Docker | Existing Desktop installation; CLI/Engine 29.8.0, Compose v5.5.1 |
| WSL backend | Existing WSL 2 configuration; WSL 3.0.3.0, kernel 6.18.54.1-1 |
| Container | Digest-pinned Node 24 Debian slim, Node 24.22.2, UID 1000(node) |

The initial audit confirmed Rust, MSVC and SDK were missing. Official rustup installed stable MSVC with Cargo, rustfmt and Clippy. The first Build Tools RunAs attempt reported cancellation without displaying a prompt to the user. Direct launch of Microsoft's signed installer installed the C++ workload and SDK successfully. Existing Node, WebView2 and Docker installations were retained. No reboot was needed to complete the observed build and UI tests.

## Native UI feasibility results

These tests used the actual Windows Tauri executable and ChatGPT website. Input was entered through the native composer using desktop UI controls, without APIs, website script injection, login or challenge bypass.

| Test | Result | Actual evidence |
|---|---|---|
| Tauri application launches | PASS | npm run tauri dev compiled and ran target/debug/chatgpt-island.exe twice |
| Private WebView2 opens | PASS | Popup visible at approximately 900 x 650; native profile check returned true; accessibility reports InPrivate |
| ChatGPT website loads | PASS | https://chatgpt.com/ rendered native composer, guest page and Log in / Sign up controls |
| Guest access | PASS | No login performed; guest conversation produced a response |
| Question submission and response | PASS | Entered the non-sensitive phrase test and arithmetic question; ChatGPT answered 4 and acknowledged the phrase |
| Follow-up context | PASS | Asked what test phrase was given; response reproduced island violet telescope 731 |
| Duplicate popup request | PASS | Launcher displayed A popup is already active or being destroyed; existing popup remained single and controls usable |
| Launcher Destroy popup | PASS | Window disappeared, terminal emitted destruction diagnostic, state became idle |
| Native X closure | PASS | Second popup disappeared and launcher returned to idle |
| Browser resource release | PASS | App-profile WebView2 process count dropped from 6 to 0 after first destruction; also 0 after native X and app shutdown |
| Fresh session on reopening | PASS | Second, third and post-restart fourth profiles opened at the empty guest home page, without previous chat content |
| Prior-session memory probe | PASS | In the second session, ChatGPT said it did not see an earlier test phrase in available conversation history |
| Shutdown with popup active | PASS | Closing launcher ended application and popup processes; process counts were 0 |
| Restart isolation | PASS | Restarted app opened a fourth fresh profile and empty conversation |
| Remote privilege configuration | PASS | Compiled ACL grants only three launcher commands to local main; no remote grants/plugins; native commands check invoking window label |
| Automatic profile deletion | NOT TESTED | Not implemented; directories are deliberately retained |
| Offline / missing-runtime fault injection | NOT TESTED | No network or installed-runtime removal was performed |
| Adversarial IPC penetration test | NOT TESTED | Security conclusion above is configuration/source review plus successful native ACL compilation, not an exploit test |

The website displayed its own response-comparison dialog during the follow-up. The visible response correctly reproduced the phrase; no preference feedback, login or account action was submitted.

## Session observations

| Opening | Distinct application-owned directory | Observation |
|---|---|---|
| 1 | island-38638ae2-caaa-415a-86b9-d314f2ea161d | Guest question, answer and follow-up context; destroyed using launcher |
| 2 | island-80644e3c-3fed-40dd-b4c6-b176de0bb276 | Empty home, new guest question reported no earlier phrase; closed using native X |
| 3 | island-e309f1bc-36ba-414d-9ba6-8a5f7039be7f | Empty home; launcher shutdown with popup active |
| 4, after restart | island-56fe3c51-7510-42aa-846e-7283814fa155 | Empty home and private-mode success; launcher shutdown |

Directories reside under the application's own LocalAppData private-sessions root. Only directory names and process ownership were inspected; no cookies, credentials or profile content were extracted. Empty UI state and the memory probe support observed isolation; unique identifiers alone were not treated as proof. These observations do not establish remote erasure or forensic local erasure.

## Commands and verification

| Command | Result |
|---|---|
| rustc --version; cargo --version; rustup show | PASS; versions above |
| node --version; npm --version | PASS |
| npm run tauri info | PASS; all native environment checks green |
| docker --version; docker compose version; docker info | PASS after starting existing Docker Desktop |
| docker compose config --quiet | PASS |
| docker compose build | PASS; npm ci installed locked dependencies |
| docker compose run --rm dev | PASS; typecheck, 2 JS tests and frontend production build |
| docker compose run --rm dev npm test | PASS; 2 tests |
| docker compose run --rm dev npm run build | PASS |
| docker compose run --rm dev id | PASS; uid=1000(node) |
| npm run typecheck; npm test; npm run build | PASS on Windows |
| cargo check --locked | PASS after Build Tools installation |
| scripts/verify.ps1 | PASS; all steps below succeeded |
| cargo test --locked | PASS; 5 Rust unit tests; main/doc tests have no cases |
| cargo fmt --check | PASS |
| cargo clippy --locked -- -D warnings | PASS |
| npm run tauri dev | PASS; native GUI launch and restart |

Initial cargo attempts failed with link.exe not found while Build Tools was absent/incomplete. The first Docker script run failed with Missing script: build\r because of Windows line endings; converted the shell script to LF, added .gitattributes and reran successfully. Those initial failures are resolved, not hidden.

## Privacy and cleanup investigation

The resolved Wry source forwards incognito to SetIsInPrivateModeEnabled but contains an older-environment fallback without private options. The app now reads the native profile IsInPrivateModeEnabled flag through Tauri with_webview and rejects sessions whose private mode cannot be verified. No page content is read for this check.

Wry closes the controller when its WebView is dropped. Process inspection confirmed all popup-profile WebView2 processes exited in the observed tests. That observation is not a general process-lifetime guarantee on every machine, nor a race-free stale-directory deletion protocol. Automatic cleanup was therefore not added. Never delete files while WebView2 may be using them. Exit the app, verify no processes associated with the exact application-owned paths remain, and delete only those paths if performing offline cleanup. If ownership or release is uncertain, preserve files. Crash leftovers are never reused.

Retained directories may contain runtime metadata or other residual files. No app conversation-storage feature exists, but complete local erasure is not promised. Closing, private mode, distinct profiles, local deletion and remote service retention are separate properties.

## Known limitations and remaining tests

- Chromium emitted `Failed to unregister class Chrome_WidgetWin_0. Error = 1412` during both launcher shutdowns. App exit status was 0 and app/popup browser process counts were 0 afterward. No orphan or crash was observed; the diagnostic remains documented.
- Top-level navigation permits only exact HTTPS chatgpt.com on port 443; new windows and downloads are denied. Other-origin login or external links may be blocked. No restrictions were weakened.
- Guest access is dependent on service, location and network. No challenges occurred during these tests.
- No signed-in browser profile was inspected. Guest controls and successful private-mode checks showed no inherited account in the tested popup; a separate signed-in-browser comparison was not performed.
- Repeated conversation creation was exercised in the first two profiles; the third and fourth verified empty state, private mode and lifecycle only.
- Fault injection for offline networking, missing runtime, locked profile files, crash recovery and adversarial IPC remains NOT TESTED. These reliability extensions do not replace the passed core guest-feasibility observations.

For reproduction, follow docs/DEVELOPMENT.md and the documented non-sensitive question/follow-up sequence above. Milestone 02 was not started.

## Subsequent milestone

This document preserves Milestone 01 observations. Milestone 02 implementation and its current native verification status are recorded in MILESTONE02.md and PROGRESS.md.
