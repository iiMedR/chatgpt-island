# ChatGPT handoff — Milestone 04

Status: **PARTIAL — implementation and native draft preparation verified; full manual submission/reply and remaining platform acceptance were deliberately not repeated.** Milestone 05 has not started.

## Implemented behavior

M3 captures and validates the explicit selection before popup focus. The lifecycle creates a fresh verified-private WebView, then starts a session-bound handoff. Its simple editable draft preserves selected text, Unicode and line breaks and leaves space for the user's question. No selection means no handoff. There are no API keys, OAuth, custom chat components, backend, undocumented endpoints or programmatic submissions.

Rust checks the native current URL and the adapter checks exact top-level origin plus the root path. About:blank can wait; login paths, query/fragment documents, unrelated origins and detected password/challenge surfaces are refused. The adapter probes the unique visible enabled editable composer for a bounded 15 seconds. It targets desktop `#prompt-textarea` / `textarea[name="prompt-textarea"]` or mobile `#mobile-composer-prompt`. These are website assumptions, not guaranteed permanent selectors. No conversation history or document text is scraped.

The textarea branch uses the browser's value setter and dispatches input. The contenteditable branch uses plain-text browser editing (`execCommand('insertText')`), whose continued compatibility is not guaranteed. JSON serialization keeps the draft data separate from executable code. An existing nonempty composer is preserved. Rust claims insertion once and never retries an ambiguous result, rerender or navigation. It verifies the resulting composer and an enabled Send control once after 300ms; the user alone submits. Send selectors include current data-testid/id controls and observed English Send prompt/Send message labels. Localized or changed layouts can require an adapter update.

If readiness/insertion/recognition cannot be confirmed, the browser stays open and a trusted local notice displays the prepared draft. It warns that a draft may already be present. The user deliberately selects/copies and pastes once; copying replaces existing clipboard formats by the user's normal Ctrl+C action. Island does not write the clipboard automatically or copy unrelated clipboard data for handoff. No bypass of login, CAPTCHA, browser security or service restrictions is provided.

Confirmed close cancels handoff work, destroys the notice and WebView, and clears the M3 selection. Cancel preserves the browser and draft. Stale session identifiers cannot affect the new popup. Source selection, temporary formatted strings and notice content remain memory-only from Island's perspective; no secure forensic wipe is promised. Application-owned profiles remain retained until safe offline cleanup, and are never reused.

## Focused verification on 2026-11-09

Testing was kept short at the user's request: one automated pass, one native smoke workflow, then one targeted correction check after that workflow found the responsive mobile composer.

- `cargo check --locked`: passed during implementation.
- `cargo test --locked`: **30 passed**, no failures, including handoff formatting/serialization/origin/exactly-once cancellation and scaled/negative/small monitor layout cases.
- `cargo clippy --locked -- -D warnings`: initially found six collapsible-if style issues; `cargo clippy --locked --fix --allow-dirty --allow-staged -- -D warnings` fixed them and completed successfully. Formatting applied. Test suite was not repeated for those style-only fixes.
- `docker compose build` and `docker compose run --rm dev`: **4 JS/security tests passed**, TypeScript and production build passed.
- The native smoke workflow initially missed ChatGPT's 520px mobile composer. It failed safely to the manual notice with the exact prepared text; clipboard was not automatically changed.
- Observed UIA composer id `mobile-composer-prompt`; added that candidate and current Send message label. Rebuilt and checked only that correction. The real private ChatGPT composer received the complete English/French/Arabic/multiline/emoji draft exactly once. Send became enabled. Appending “Explain this to me simply.” preserved the original draft. **No message was submitted.** The correction was compiled and exercised natively; no full test/compatibility suite was repeated.
- Popup screenshot showed the real ChatGPT page filling a compact borderless window, no native title-bar controls/address bar/tabs/custom header, rounded native edges and normal editable input. Native private-mode verification passed on creation. Accessibility may still expose synthetic title-bar/system controls even though they are not visible.

| Test | Status |
|---|---|
| Opera selection → ChatGPT draft | NOT TESTED in M4; capture-only passed in M3 |
| VS Code selection → ChatGPT draft | NOT TESTED; default editor capture failed in M3 |
| Notepad selection → ChatGPT draft | PASS after mobile-composer correction |
| Draft editable; extra typing preserves selection | PASS |
| No automatic submission | PASS in observed workflow; Send never activated |
| Guest ChatGPT responds / follow-ups | NOT TESTED in M4; historical M1 response evidence remains separate |
| Missing composer → trusted manual notice | PASS in initial mobile-selector failure |
| Borderless popup / native rounded edges | PASS visually on development host |
| Keyboard move/resize and native mouse resize | NOT TESTED; implemented native mechanisms |
| Escape/shortcut close and Cancel | Existing M2 behavior retained; not rerun in M4 |
| Fresh session after close | Fresh private profiles/mode checks retained; real conversation isolation not rerun in M4 |
| Windows 10, second monitor, mixed DPI, hot unplug | NOT TESTED natively; geometry policy has unit coverage |

## Limits and privacy

Insertion places selected text in a third-party page where its scripts can read it, before Send. It is not a submitted message. Island itself performs no hidden network requests for handoff and does not inspect cookies/tokens. Website-side draft/network processing was not instrumented and cannot be guaranteed absent. If unexpected pre-submission transmission is observed, discontinue automatic insertion and investigate; user-assisted handoff remains available.

The current mobile textarea path was verified. Desktop contenteditable, localized Send labels, slow/offline loading, unusual overlays and later website changes are not claimed to work universally. Short callback timeouts or slow loading can fall back despite an eventually usable page. The native rounding API chooses the radius; exact 10px is not claimed. Shadow appearance depends on OS/compositor. Window changes are handled on movement/resize/scaling events; monitor hot-plug recovery and keyboard system commands need separate platform validation. The M2 physical tray Quit delivery issue remains unverified and is not claimed fixed.

Milestone 05 would cover reliability, remaining platform/compatibility cases and Windows packaging/installer. It was not begun here.

Final integration note: fallback-notice creation is serialized through the lifecycle queue so a closing session cannot leave a late notice over a new popup. Final `cargo check --locked` passed after that adjustment; the test suites were not repeated. Windows incremental-cache hard links fell back to copying without a compilation failure. The sample question remains an unsent draft in the native popup for optional user review.
