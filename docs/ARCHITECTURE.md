# Architecture

ChatGPT Island is a Windows-only Tauri 2 / Rust utility. Wry hosts Microsoft WebView2. The remote popup contains actual https://chatgpt.com; trusted Vanilla TypeScript is used only for confirmation, settings and an optional development launcher.

## Native modules

| Module | Responsibility |
|---|---|
| main.rs / lib.rs | Runtime/plugin initialization, tray startup, native window events, background lifetime and authorized exit |
| tray.rs | Official TrayIconBuilder, title/Open/Settings/Quit menu, left-click open/focus |
| shortcuts.rs | Official global-shortcut plugin, Alt+Shift+E registration, press/release latch and registration cleanup state |
| lifecycle.rs | Typed state model, one worker queue, popup/session ownership, decision validation, local commands and orderly Quit |
| popup.rs | Existing incognito/unique-directory WebView construction, private-mode verification and native Escape handler |
| session.rs | Atomic unique profile creation and restricted URL policy |
| confirmation.rs | Creation/focus of trusted local windows and local-origin navigation restrictions |
| settings.rs | Default/corruption handling and reliable single-boolean preference persistence |

## Background lifetime

Normal configuration has no startup windows. Setup installs the lifecycle controller, creates the tray and registers the shortcut once. The optional --dev-launcher flag is accepted only by debug builds. Tray initialization failure aborts startup instead of leaving an inaccessible invisible process. Shortcut failure keeps the tray usable and opens Settings with conflict diagnostics.

The runtime prevents automatic exit when the last popup/local window closes. Only the lifecycle's authorized Quit sets the exit gate. Quit unregisters the shortcut, exits the runtime and ends the worker. There is no autostart or continuous application scanning. Capture and optional clipboard access occur only on the explicit shortcut. Normal idle execution blocks on the worker queue; only the explicitly enabled development launcher polls status.

## Lifecycle

The typed states are Closed, Opening, Open, AwaitingCloseConfirmation and Closing.

```text
Closed -> Opening -> Open -> AwaitingCloseConfirmation
                       ^          | Cancel
                       +----------+
                                  | Confirm
                                  v
                               Closing -> Destroyed -> Closed
Open -- warning disabled ------> Closing
```

A successful open reserves a UUID session/profile and moves to Opening before calling native creation. Success verifies private mode and installs native input handling before moving to Open. Failure releases ownership if no window exists; a live failed window remains reserved until actual destruction. No second popup can be opened during Opening or Closing.

Tray Open focuses an existing popup. Shortcut toggles request closure. Native X and Escape request closure through the same controller. AwaitingCloseConfirmation prevents duplicate dialogs; repeated requests focus the one existing confirmation. Cancel restores Open without changing the session. Confirm transitions to Closing, calls destroy rather than hide, and waits for Destroyed before clearing the profile reference. Stale session events and confirmation decisions are rejected/ignored by identifier checks.

Quit records pending exit. If confirmation is required, Cancel also cancels Quit. Confirm destroys the popup; actual destruction then completes exit. Quit without a popup exits directly. Force termination, crashes and OS shutdown may bypass the warning.

## Threading and input

Native callbacks only enqueue requests. A single named worker serializes effects. It holds no lifecycle mutex during WebView creation, UI dispatch, settings I/O or native destruction. Tauri dispatches WebView operations to the owning event loop. Private-mode and Escape-handler attachment use bounded worker-side waits; the UI thread never waits on this queue.

The shortcut plugin handles only Alt+Shift+E. A pressed-state latch accepts the initial Pressed event, ignores key repeat, and resets on Released. No arbitrary debounce timer or low-level global keyboard hook is used. Unregistration is idempotent and also attempted on runtime exit.

Escape uses the popup's WebView2 controller AcceleratorKeyPressed event. Only Escape keydown is handled; WasKeyDown suppresses repeat. The callback enqueues a close request and captures no WebView reference. Controller destruction releases the COM event handler. Escape is not globally registered and no website JavaScript is injected. It takes precedence over website menu/dialog Escape interactions; this compatibility tradeoff is documented.

## Trusted UI and settings

The confirmation window has a dedicated local capability permitting only resolve_confirmation. Its URL carries the session identifier; Rust rejects a decision for any other/non-pending session. Native X or local Escape cancels the dialog. Cancel is the initial keyboard focus. The optional checkbox is committed only after an affirmative Close conversation decision.

The settings window can read the safe snapshot and reset the warning. Each role is separated from the development launcher capability. Local windows restrict navigation to packaged app origins (and the exact loopback Vite origin in debug). New windows are denied.

Only skip_close_confirmation and clipboard_fallback_enabled are serialized; both default to false. Unknown/corrupt data defaults to warnings enabled. Writes use a unique temporary file in the app configuration directory, flush it, then replace settings.json with Windows MoveFileExW replace-existing/write-through flags. Persistence failure leaves the conversation alive and reports the error. There is no database, message history, cookie capture or token access. Separately launched processes load preferences independently; live cross-process preference synchronization is not implemented.

## WebView2 privacy and cleanup

The existing Tauri builder uses External(ChatGPT URL), incognito(true), a fresh application-owned data directory, 520 x 660 logical-pixel resizable borderless windows placed against the source monitor working area, and disabled developer tools. Native ICoreWebView2Profile::IsInPrivateModeEnabled must return true; older-runtime fallback without private mode is rejected.

Profile identifiers and paths are owned by the active lifecycle record and invalidated on Destroyed. Every new opening atomically creates another directory; prior and crash-leftover profiles are never reused. No automatic deletion is performed because window/controller destruction alone does not establish browser-process exit on every host. Retention is preferred over deleting locked or ambiguously owned paths. This neither promises complete local erasure nor remote ChatGPT deletion.

## Security boundary

The remote popup has no capability, even though the Rust-side application uses tray and shortcut APIs. No global-shortcut JavaScript permissions are granted. Local roles have exact labels, no wildcard windows, no remote origins, and only the commands each role needs. Every command additionally validates its invoking window label. The settings and confirmation pages never render diagnostics as HTML.

Top-level popup navigation permits only exact HTTPS chatgpt.com on port 443 with no embedded credentials. New windows and downloads are denied. Website HTTPS subresources are not filtered by this top-level policy. No broad native, filesystem, shell or settings privileges reach ChatGPT. Standard Tauri origin checks and launcher CSP remain enabled. Authentication, CAPTCHA and rate-limit bypasses are not implemented.

## Development split

Docker uses the existing digest-pinned Node 24 image, npm ci and non-root containers for TypeScript, static security/configuration tests and frontend builds. No mounts, browser profiles, Docker socket or published ports are required. Native Windows verification covers Rust MSVC compilation/tests/Clippy plus actual tray, shortcut and WebView2 behavior. Cargo.lock pins native dependencies. Docker cannot validate Windows keyboard delivery or GUI integration.

References: [Tauri tray API](https://docs.rs/tauri/2.12.5/tauri/tray/struct.TrayIconBuilder.html), [official shortcut plugin](https://v2.tauri.app/plugin/global-shortcut/), [Tauri capabilities](https://v2.tauri.app/reference/acl/capability/).


## Selected-text capture (Milestone 03)

Only a fresh Alt + Shift + E press in Closed initiates capture. The shortcut callback snapshots foreground HWND/PID before enqueueing; tray/local Open does not capture. The lifecycle worker reserves a unique session, completes capture and validation, stores one pending selection for that session, then creates/focuses the private WebView. Presses while open follow the existing close path without capturing. No-selection/error opens normally with content-free diagnostics in Settings.

`text_capture/coordinator.rs` launches the same executable in a windowless helper mode. HWND/PID and opt-in are arguments; text is returned only over an anonymous stdout pipe, capped at 96 KiB. Terminal/file stdout is refused. The helper initializes COM MTA on its non-UI thread, uses CUIAutomation8 and requests 250 ms connection/500 ms transaction timeouts. A parent deadline of 1,500 ms kills/waits for a stuck helper; no detached provider thread survives. Provider work can delay lifecycle commands by that bound; process startup/termination adds OS overhead. The automated stalled-child test exercises actual termination, rather than relying on COM honoring timeouts.

UIA starts at the focused control and ascends at most eight ancestors in the originating PID, stopping at the foreground window. It checks password indicators and the default input desktop, and reads at most 32 noncollapsed GetSelection ranges. It never reads DocumentRange, GetText(-1), other applications, page DOM or application history. Range reads are bounded and combined with newlines. Inaccessible/unknown protected properties fail closed. Providers that misreport passwords or selections remain an OS/application limitation.

Validation trims outer whitespace, preserves internal Unicode/newlines, rejects empty/whitespace/NUL and selections over 10,000 Unicode scalar values. Oversized text is rejected, not silently truncated. Unicode scalar counts are not grapheme-cluster counts.

Clipboard fallback is disabled by default and persisted only through the trusted Settings command. It requires a positively identified noncollapsed UIA range whose text read failed, plus an initially empty clipboard. Unsupported UIA and collapsed/no-selection never trigger blind Ctrl+C (some editors copy a whole line without a selection). Existing clipboard formats are counted under a clipboard lock but never read, serialized, cloned or replaced. The helper waits up to 300 ms for shortcut modifiers to release, verifies foreground and sequence, sends Ctrl+C, and waits up to 300 ms for new Unicode data. It checks source ownership and sequence, bounds the locked memory read, closes all clipboard handles, and restores the initial empty state only while the clipboard lock proves sequence/owner unchanged. It leaves external changes untouched. This strategy deliberately sacrifices compatibility instead of pretending rich clipboard formats can be restored from plain text.

Clipboard operations cannot be atomic with focus/other-app actions. A change between the final foreground/sequence check and SendInput can still race; delayed rendering or timeout can leave a copy on the clipboard. The trusted Settings window warns when copy/restoration may have failed. Clipboard managers, cloud/history features and source apps may retain a copy; Island cannot retract it. The code never bypasses UAC, protected desktops or input integrity restrictions.

`PendingSelection::for_session(id)` is the narrow Rust interface for future handoff. New sessions replace the single value. Active Destroyed, popup-open failure and application exit release it; Cancel and stale destruction events preserve the active selection. Text is not Debug/Serialize on application state, is not included in settings or diagnostic snapshots, and is never written to application logs/files or sent to ChatGPT. Normal memory release is not a forensic secure-wipe/pagefile guarantee. An explicit debug-only `--dev-selection-preview` flag permits fixture previews through the main local window; production rejects that command and the remote popup has no capability. Milestone 03 contained no insertion/submission; the Milestone 04 draft-only handoff is described below.

References: [Microsoft UI Automation threading](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-threading), [GetSelection](https://learn.microsoft.com/en-us/windows/win32/api/uiautomationclient/nf-uiautomationclient-iuiautomationtextpattern-getselection), [Windows clipboard](https://learn.microsoft.com/en-us/windows/win32/dataxchg/clipboard).

## Milestone 04 handoff and popup

`handoff/mod.rs` owns a typed session-bound Pending → WaitingForChatGPT → Inserting → Ready/Failed state with an atomic cancellation token and a single insertion claim. It consumes M3's `PendingSelection::for_session` interface; it never re-enters Windows capture APIs. The coordinator runs on a bounded worker, checks current session and lifecycle before work, pauses while close confirmation is pending, and stops on confirmed close, destruction or exit. Cancel retains selection, page draft and browser state. A timed-out paused handoff becomes Failed; Cancel can show its manual notice. A new profile cannot receive a previous session's callbacks.

`src/composer-script.js` is a small synchronous per-attempt adapter, embedded by Rust rather than loaded as persistent page automation. Supported Tauri `eval_with_callback` uses WebView2 script execution. Rust validates the native URL, then script rechecks top-level exact `https://chatgpt.com` origin, root path with no query/fragment, document readiness, visible unique editable composer and protected/challenge indicators. Login/other-origin/other-path documents receive no insertion. The initial blank page is waited out. Probes contain no selected text, use 300ms intervals within a 15-second deadline, and each native callback waits at most 750ms. No MutationObserver, navigation listener, Tauri bridge or injected host objects are installed in the website.

Compatibility-sensitive candidates are `#prompt-textarea`, `textarea[name="prompt-textarea"]` and the observed mobile `#mobile-composer-prompt`. An occupied composer is never overwritten. A textarea uses its native value setter and an input event; contenteditable uses browser `execCommand('insertText')` with a plain-text argument to keep editing/undo behavior. The latter is a compatibility-sensitive legacy browser editing API, not a permanent ChatGPT integration contract. Untrusted text is serialized with serde_json as data, never interpolated as executable statements or HTML. One insertion is followed by one delayed check of composer text and an enabled Send control; only short status strings return to Rust. User edits, missing controls, execution errors and uncertain results fail without an automatic retry. No click, Enter, submit, fetch, cookies, token access or undocumented endpoint calls exist in the adapter.

A failed handoff preserves the M3 selection and opens the exact-label local `handoff` notice, with its own two-command capability. It can read the active session draft and focus that popup; it cannot initiate capture. The notice renders draft text as a textarea value, not HTML. Select draft plus user Ctrl+C is the only fallback copying path; no automatic clipboard replacement or new clipboard permission is introduced. The notice is destroyed with its owning session. The external popup retains zero native capabilities. Remote document script execution does not grant it a native IPC permission.

`window_layout.rs` selects the source HWND's monitor or cursor monitor, reads physical working-area bounds/effective DPI, and clamps initial dimensions/minimums to the available space. Negative monitor coordinates are retained. Creation stays hidden until private-mode verification, sizing, native corner preference and Escape handling are installed. The 520×660 logical target and 380×420 minimum shrink on smaller working areas. Moved/Resized/ScaleFactorChanged events constrain the popup to the nearest current working area. Windows 11 DWM rounding is best-effort; Windows 10 can retain square corners. Native resizable borders/shadow remain, with no transparent/click-through region or mouse-selection overlay. Alt+F7/F8 posts native move/size system commands only while the popup's WebView has focus. Escape retains M2's documented priority over website Escape menus.

Preparing website input exposes it to that site's JavaScript. No submitted-message action or app handoff request occurs, but website-side processing before submission cannot be guaranteed absent. Private mode, no app persistence, memory release and remote retention are distinct properties. Profile directories remain unique and retained for safe offline cleanup; no unsafe deletion protocol was added.

References: [Microsoft WebView2 security](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/security), [WebView2 script execution](https://learn.microsoft.com/en-us/microsoft-edge/hosting/webview2/gettingstarted), [native rounded corners](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/ui/apply-rounded-corners).
