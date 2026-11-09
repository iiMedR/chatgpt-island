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

The runtime prevents automatic exit when the last popup/local window closes. Only the lifecycle's authorized Quit sets the exit gate. Quit unregisters the shortcut, exits the runtime and ends the worker. There is no autostart, global text capture, clipboard handling or continuous application scanning. Normal idle execution blocks on the worker queue; only the explicitly enabled development launcher polls status.

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

Only skip_close_confirmation is serialized. Unknown/corrupt data defaults to warnings enabled. Writes use a unique temporary file in the app configuration directory, flush it, then replace settings.json with Windows MoveFileExW replace-existing/write-through flags. Persistence failure leaves the conversation alive and reports the error. There is no database, message history, cookie capture or token access. Separately launched processes load preferences independently; live cross-process preference synchronization is not implemented.

## WebView2 privacy and cleanup

The existing Tauri builder uses External(ChatGPT URL), incognito(true), a fresh application-owned data directory, 900 x 650 centered/resizable decorated windows, and disabled developer tools. Native ICoreWebView2Profile::IsInPrivateModeEnabled must return true; older-runtime fallback without private mode is rejected.

Profile identifiers and paths are owned by the active lifecycle record and invalidated on Destroyed. Every new opening atomically creates another directory; prior and crash-leftover profiles are never reused. No automatic deletion is performed because window/controller destruction alone does not establish browser-process exit on every host. Retention is preferred over deleting locked or ambiguously owned paths. This neither promises complete local erasure nor remote ChatGPT deletion.

## Security boundary

The remote popup has no capability, even though the Rust-side application uses tray and shortcut APIs. No global-shortcut JavaScript permissions are granted. Local roles have exact labels, no wildcard windows, no remote origins, and only the commands each role needs. Every command additionally validates its invoking window label. The settings and confirmation pages never render diagnostics as HTML.

Top-level popup navigation permits only exact HTTPS chatgpt.com on port 443 with no embedded credentials. New windows and downloads are denied. Website HTTPS subresources are not filtered by this top-level policy. No broad native, filesystem, shell or settings privileges reach ChatGPT. Standard Tauri origin checks and launcher CSP remain enabled. Authentication, CAPTCHA and rate-limit bypasses are not implemented.

## Development split

Docker uses the existing digest-pinned Node 24 image, npm ci and non-root containers for TypeScript, static security/configuration tests and frontend builds. No mounts, browser profiles, Docker socket or published ports are required. Native Windows verification covers Rust MSVC compilation/tests/Clippy plus actual tray, shortcut and WebView2 behavior. Cargo.lock pins native dependencies. Docker cannot validate Windows keyboard delivery or GUI integration.

References: [Tauri tray API](https://docs.rs/tauri/2.12.5/tauri/tray/struct.TrayIconBuilder.html), [official shortcut plugin](https://v2.tauri.app/plugin/global-shortcut/), [Tauri capabilities](https://v2.tauri.app/reference/acl/capability/).
