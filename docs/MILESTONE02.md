# Milestone 02 verification — 2026-11-09

This report distinguishes tool-observed native behavior, user-reported tray behavior and automated state/configuration checks. Full completion requires the remaining exit/process check below.

## Native Windows results

| Test | Result | Evidence |
|---|---|---|
| Background startup | PASS | Native process remained alive with no Island window or popup in the window inventory |
| Tray icon/menu availability | PASS | User opened Settings from the tray and confirmed it stays open on the stable build |
| Tray Settings | PASS | User-reported successful tray launch; trusted Settings was separately inspected during conflict testing |
| Global shortcut from another app | PASS | Alt + Shift + E sent with Docker Desktop focused opened one private ChatGPT popup |
| Shortcut close request | PASS | Same shortcut opened the local confirmation while keeping the popup alive |
| Popup opening/private mode | PASS | Real ChatGPT page rendered; native profile check succeeded; accessibility reported InPrivate |
| Guest functionality preserved | PASS | Native composer accepted a non-sensitive phrase test and ChatGPT responded |
| First-close confirmation | PASS | Trusted local window displayed warning, checkbox, Cancel and Close conversation |
| Cancel preserves conversation | PASS | Original window and conversation URL/test response remained after Cancel |
| Checked Cancel does not persist | PASS | Visibly checked checkbox was canceled; settings file remained absent; warning appeared again |
| Confirm destroys popup | PASS | Window disappeared, popup browser process count became 0, background process remained |
| Fresh guest session | PASS | Reopening showed empty guest home without earlier test conversation |
| Native X | PASS | X requested confirmation rather than destroying immediately |
| Focused Escape | PASS | Native Escape produced the same local confirmation |
| Escape outside popup | PASS | Escape in Docker Desktop did not close the Island popup or show confirmation |
| Duplicate confirmation prevention | PASS | Repeated shortcut while awaiting a decision left one dialog and one popup |
| Preference save and skip | PASS | Confirming a visibly checked checkbox wrote only skip_close_confirmation=true; next close destroyed without warning |
| Preference persistence | PASS | A separately launched process loaded Disabled from the saved setting |
| Settings reset | PASS | Reset changed UI to Enabled and settings.json to skip_close_confirmation=false; subsequent warning appeared |
| Shortcut conflict | PASS | Second test instance reported already-registered shortcut and remained usable through trusted controls |
| Native application close with no popup | PASS | Closing the debug launcher ended the conflict-test instance; original background instance remained |
| Tray Open and active-popup Quit | NOT TESTED | User reported Open, Cancel and confirmed Quit succeeded, but process inspection still found one background process; exit is being investigated |
| Shortcut release after orderly Quit | NOT TESTED | Requires confirmed process exit and shutdown trace |

The non-sensitive test phrase used in the first guest conversation was island amber kite 204. No credentials, cookies, tokens or other browser-profile contents were read. Only application process counts, app-owned profile-directory names and the single boolean settings file were inspected.

A development rebuild coincided with the user's first tray test and closed/restarted the development application. Later stable-build user feedback confirmed Settings stayed open. One private-mode verification was interrupted during development shutdown (the callback channel closed); this is recorded as an interrupted test rather than a successful session or proven runtime defect.

## Automated verification

The final verification before adding shutdown-only trace messages passed:

- docker compose build
- docker compose run --rm dev: TypeScript, 2 JavaScript configuration/security tests and production frontend build
- scripts/verify.ps1: Windows TypeScript, JavaScript tests, frontend build, cargo check --locked, cargo test --locked, cargo fmt --check, cargo clippy --locked -- -D warnings
- Rust: 15 passing tests covering lifecycle, stale identifiers, duplicate/closing protection, canceled Quit, actual-destruction requirement, creation failure recovery, URL restrictions, unique profile allocation, settings defaults/corruption/persistence, checkbox policy, pressed-event latching and mocked registration/cleanup failure handling

Compilation initially caught a duplicate PathBuf import introduced during module organization; it was fixed and all native checks passed afterward. Automated tests establish state and configuration behavior, not native tray or keyboard delivery.

## Security and privacy

Remote ChatGPT content receives no Tauri capability. The optional launcher, local confirmation and Settings each have exact separate permissions and additional Rust label checks. The confirmation permission is only resolve_confirmation; Settings can read a safe snapshot and reset the warning. Frontend code cannot register shortcuts or access filesystem/shell functions. Trusted windows restrict navigation to exact packaged origins and the debug loopback origin.

Escape is intercepted only by the popup's native WebView2 accelerator event and suppresses key repeat. It takes precedence over ChatGPT's own Escape handling, including website menus/dialogs; Cancel preserves the conversation. No global Escape hook or privileged webpage script is used.

Runtime profile verification and unique directories are preserved. Lifecycle ownership ends only after the actual Destroyed event. Profile directories are retained because a general safe browser-process-exit cleanup protocol is still not implemented. Crash leftovers are never reused. Retention avoids deleting active or ambiguously owned data, but does not promise full local erasure. Closing does not delete remote ChatGPT server data.

## Remaining checks and limitations

- Resolve the remaining background process after the user-reported Tray Quit; do not mark the milestone complete until confirmed.
- Physical tray actions require user help because the available native automation surface does not expose the taskbar.
- Computer Use was stopped by physical Escape during testing; no further native automation was performed after that stop in that turn.
- Offline/missing-runtime fault injection, crash recovery, profile deletion and adversarial IPC testing remain unverified.
- Multiple separately launched processes load settings independently; preferences are not synchronized live across processes.
- No Milestone 03 text capture, Milestone 04 handoff/polish or Milestone 05 installer work was implemented.

Shutdown-only diagnostics were added after the remaining process was observed. The final Rust tests (15), formatting and strict Clippy passed again. At the final check, no new tray Quit request had appeared in the diagnostic log and the background process remained idle; the requested final user action is still pending.

## Tray dispatch follow-up

The user selected Quit again, but the inspected process remained alive and no Quit request appeared in the lifecycle log. Installed Tauri tray ownership/event registration was reviewed. Added a content-free trace of application menu identifiers and explicit reporting if enqueueing a tray command fails. Native compilation, all 15 Rust tests and strict Clippy pass. The final diagnostic build is running; confirmation of the exact physical tray menu and a new diagnostic trace is pending. This is not recorded as a successful Quit or a fixed defect.
