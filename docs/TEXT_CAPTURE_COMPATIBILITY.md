# Text capture compatibility — Milestone 03

Status: **PARTIAL — implemented and verified in supported controls; remaining native coverage is unverified.** Native testing stopped at the user's request after the results below. No additional compatibility passes are implied.

## Observed Windows results (2026-11-09)

Tests used non-sensitive sample text, the real Alt + Shift + E shortcut, a debug-only trusted preview, and private WebView2 popups. Clipboard fallback stayed disabled. These results describe the tested controls/configurations, not every version or document in an application.

| Application | UI Automation | Clipboard fallback | Result |
|---|---|---|---|
| Windows Notepad | PASS | NOT TESTED | Selected first line only; English, French accents and Arabic preserved; second unselected line excluded. No-selection opened normally in 94 ms. |
| Opera | PASS | NOT TESTED | Sample textarea: three lines, French/Arabic/emoji preserved; 53 Unicode scalars, 133 ms. Dummy password selection rejected in 133 ms. |
| Google Chrome | PASS | NOT TESTED | Same multiline fixture captured exactly; 53 Unicode scalars, 127 ms. |
| Microsoft Edge | PASS | NOT TESTED | Same multiline fixture captured exactly; 53 Unicode scalars, 116 ms. |
| Visual Studio Code | FAIL in tested default editor | NOT TESTED | Visible first-line selection returned no selection through accessibility; popup opened normally without context in 167 ms. No blind copy attempted. |
| Microsoft Word | NOT TESTED | NOT TESTED | Installed. Launch was attempted; Computer Use reported Word was not approved. No document/capture result obtained. |
| PDF reader | NOT TESTED | NOT TESTED | No native PDF selection result obtained before testing stopped. |

Measurements are single observed helper/capture durations, including helper startup/IPC; they exclude website load time and are not percentiles or performance guarantees. First Notepad selected-line latency was not recorded. Native private-mode verification succeeded on popup creation. Existing skip-warning preference allowed test popup destruction without a dialog. Closed-state preview contained no stale selection; Rust tests additionally cover Cancel, active/stale destruction and pending-state replacement.

## Automated results already obtained

- `docker compose build` and `docker compose run --rm dev`: TypeScript check, 2 security/configuration tests and production build passed.
- `cargo check --locked`: passed.
- `cargo test --locked`: 26 passed, 0 failed. Includes Unicode/limits, UIA success/unavailability, disabled/enabled fallback, copy/preservation/restore failures, password/foreground/timeout rejection, session replacement/clearing, lifecycle integration, preference migration and settings serialization containing only two booleans.
- A real stalled child process was terminated by the 1,500 ms watchdog; test suite completed in 1.56 seconds. Provider outcomes and clipboard failure cases otherwise use mocks/pure policy tests, not a deliberately hung native accessibility provider.
- `cargo fmt` and strict Clippy passed during implementation. Final formatting/compile status is recorded in PROGRESS.md.
- Initial sandboxed settings test failed because Windows denied atomic replacement. The normal-context native run passed; atomic writes were preserved.

## Limits and remaining work

The clipboard fallback is intentionally narrower than a general copy workaround: it needs a UIA-confirmed noncollapsed selection with a failed text read and an initially empty clipboard. Unsupported text patterns and no-selection never cause Ctrl+C. This prevents accidental whole-line/document copying. Existing text, HTML, rich text, images, file lists, delayed-rendered or app-specific formats cause refusal without reading/replacing them. Only an unchanged, source-owned copy may be cleared to restore the initial empty state. Foreground and clipboard changes can race between checks and input; restoration/history cannot be guaranteed. A trusted warning reports potentially retained copies.

Native opt-in UI interaction, real fallback success, complex-format preservation, clipboard-manager/concurrent-change/delayed-rendering cases, oversized native selection, elevated controls and PDF/Word remain unverified. Unit tests do not substitute for those results. Default UIA works independently of fallback. No universal Windows support is claimed. Milestone 02's physical tray Quit delivery issue remains separately unresolved; M3 does not claim to fix it.

No captured text is injected into the website, submitted, logged or persisted by Island. The local fixture file contains authored test data only. Source applications/clipboard managers can retain their own data; ordinary memory release is not forensic erasure. The explicit debug preview is for samples only and is rejected in production. See ARCHITECTURE.md for the capture/IPC/privacy boundary and DEVELOPMENT.md for optional future test steps.

This file records M3 evidence. Subsequent draft-only handoff and popup implementation is documented in CHATGPT_HANDOFF.md; M3 capture results above remain historical and were not rerun.
