# Approved final-product requirements

These describe the intended product, not completed Milestone 01 features.

- Windows-only desktop application using Tauri 2 and Rust.
- Background operation, system tray, and global Alt + Shift + E shortcut.
- Selected-text capture from supported Windows applications.
- Private popup containing the actual ChatGPT website.
- No custom AI interface, OpenAI API, OAuth, or required application signup.
- No native close/minimize/maximize buttons in the final popup.
- Approximately 10px rounded corners.
- Escape or global shortcut closes the popup.
- First-close confirmation and don't-show-again preference.
- Fresh private conversation on every opening; no persistent island conversation storage.
- Respect website authentication, guest availability, rate limits, verification, and service restrictions.

Milestone 01 uses native window decorations and a development launcher. It does not implement the tray, shortcuts, selection capture, composer insertion, preferences, final styling, autostart, installer, or distribution. Local profile residue and empirical privacy verification remain open concerns.
