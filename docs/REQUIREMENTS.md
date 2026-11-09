# Approved requirements and milestone status

## Intended final product

- Windows-only desktop application using Tauri 2 and Rust.
- Background operation, system tray and global Alt + Shift + E shortcut.
- Selected-text capture from supported Windows applications.
- Private popup containing the actual ChatGPT website.
- No custom AI interface, OpenAI API, OAuth or required application signup.
- Final popup without native close/minimize/maximize buttons; approximately 10px rounded corners.
- Escape or global shortcut requests closure.
- First-close confirmation, Don't show this again and warning reset preference.
- Fresh private conversation on every opening; no application conversation-history storage.
- Respect authentication, guest availability, rate limits, verification and service restrictions.

## Five approved milestones

1. **Foundation & Guest ChatGPT Feasibility:** verified on the tested host/network. See FEASIBILITY.md. Safe profile-retention limits remain documented.
2. **Background App, System Tray & Shortcuts:** implemented; verification is recorded in MILESTONE02.md. Includes typed lifecycle, native Escape, trusted close warning, persistent boolean preference and Settings reset. Full completion depends on verified tray Quit and shortcut release.
3. **System-Wide Text Selection:** implemented with bounded UI Automation, opt-in empty-clipboard fallback, Unicode validation and session-scoped memory. Native compatibility and outstanding tests are recorded in TEXT_CAPTURE_COMPATIBILITY.md; universal support is not claimed. The capture module itself does not insert or submit text; M4 handles draft-only preparation.
4. **ChatGPT Handoff & Popup Polish:** implemented; verification remains partial. Session-bound, origin-checked draft-only preparation, manual fallback notice and borderless monitor-aware native popup are present. Current mobile composer insertion/editing passed natively. Full manual submission/reply, desktop editor and remaining platform cases were not repeated; see CHATGPT_HANDOFF.md.
5. **Testing, Reliability & Windows Installer:** release infrastructure implemented locally. Windows x64 NSIS, final supplied branding, MIT/license notices and a tag-only draft-release workflow are configured. Clean-machine installation/upgrade/uninstall, hosted workflow execution and outstanding runtime acceptance remain unverified. No release was published; see RELEASING.md and RELEASE_BUILD_REPORT.md.

Milestone 04 replaces the historical Milestone 02 decorated popup with a borderless window; Alt+F4, Escape and the global shortcut retain close safeguards. Normal startup has no launcher; an optional debug-only launcher remains available. Autostart is not implemented. No application setting stores conversation text, selected text, cookies or authentication data. Profile directories remain application-owned and unique, but automatic deletion is not implemented.
