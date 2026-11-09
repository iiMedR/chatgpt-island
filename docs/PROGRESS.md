# Progress

1. **Foundation & Guest ChatGPT Feasibility — COMPLETE on tested host/network**
   Core website, guest conversation, private mode, destruction, reopening and restart were verified. See FEASIBILITY.md for historical evidence and retention limits.
2. **Background App, System Tray & Shortcuts — PARTIAL pending final Quit verification**
   - [x] No startup launcher or empty window; background lifetime retained.
   - [x] Official tray icon/menu and Rust global-shortcut plugin implemented.
   - [x] Shortcut opens from another app and requests confirmation when open.
   - [x] Native popup Escape and X route through the lifecycle controller.
   - [x] One-popup/one-dialog enforcement and Closing-state protection.
   - [x] Cancel preserves conversation; checked Cancel does not save preference.
   - [x] Confirm destroys popup while background process remains.
   - [x] Preference save, persistence, skip behavior and Settings reset verified.
   - [x] Real shortcut conflict reported without disabling native controls.
   - [x] Unique-profile isolation and native private-mode verification retained.
   - [x] Exact local capabilities; no privileged remote commands.
   - [x] Docker and native verification pass (15 Rust tests, 2 JS tests, formatting, strict Clippy).
   - [x] User confirmed stable tray Settings interaction.
   - [ ] Confirm actual Tray Quit process exit and shortcut release.
   - [x] Local milestone implementation commit created; no push.
3. **System-Wide Text Selection — PARTIAL (implementation complete; native coverage remains)**
   - [x] Dedicated typed Rust capture module, MTA UIA helper, bounded process watchdog.
   - [x] Capture before focus, selected ranges only, Unicode validation and 10,000-scalar limit.
   - [x] Opt-in empty-clipboard fallback; password/foreground/sequence/owner safeguards.
   - [x] One session-scoped in-memory value; destruction/failure/exit clearing; no website capture IPC.
   - [x] Native Notepad, Opera, Chrome and Edge capture verified; no-selection/password cases verified.
   - [x] Docker checks and 26 Rust tests passed; strict Clippy passed.
   - [ ] Native fallback/clipboard-manager, Word/PDF and remaining edge-case coverage. Testing stopped at user request.
   - [x] Compatibility limits and observed VS Code failure documented in TEXT_CAPTURE_COMPATIBILITY.md.
4. **ChatGPT Handoff & Popup Polish — PARTIAL (implementation and draft workflow verified)**
   - [x] Typed session-bound handoff; bounded readiness; safe JSON data; at-most-once insertion.
   - [x] Exact native/script origin and document checks; preserve occupied composers; no submission.
   - [x] Trusted manual selection/copy notice on failure; no automatic handoff clipboard write.
   - [x] 520×660 borderless popup, native shadow/corners, working-area/DPI placement and clamping.
   - [x] Native keyboard move/size mechanisms; no website drag privileges or overlay.
   - [x] 30 Rust tests and 4 Docker JS/security tests passed; frontend typecheck/build passed.
   - [x] Strict Clippy completed after six style fixes; formatting applied.
   - [x] Focused native Notepad → multilingual editable draft passed after observed mobile-selector fix; Send enabled; no submission.
   - [x] Actual failed-handoff notice and native borderless/rounded rendering observed.
   - [ ] Manual guest response/follow-up, desktop contenteditable, Windows 10/mixed-DPI/multiple monitors/hot-plug and native move/resize acceptance remain unverified.
   - [x] No repeated compatibility sweep; respected user's reduced-testing request. Details in CHATGPT_HANDOFF.md.
5. **Testing, Reliability & Windows Installer — NOT STARTED**

Detailed observations, interrupted tests and remaining verification are in MILESTONE02.md. Safe automatic profile deletion remains unimplemented. The primary isolation behavior uses fresh, verified-private WebViews; remote erasure is not claimed.


Final M3 verification: formatting applied and `cargo check --locked` passed after the final clipboard error/transport guards. Windows reported incremental-cache hard-link fallback to copying; compilation succeeded. No further test-suite or native UI runs were made after the user requested testing stop. The 26-test and Docker results above refer to the completed earlier runs. No captured user data or runtime artifacts are included in the commit.

Final M4 integration compiled successfully after serializing fallback notices with popup destruction. No repeated suite or compatibility sweep was performed. Full implementation/evidence/limits are in CHATGPT_HANDOFF.md. Milestone 05 remains unstarted.
