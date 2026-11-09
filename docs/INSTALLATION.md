# Installing ChatGPT Island on Windows

ChatGPT Island is an independent project, not affiliated with, endorsed by, or sponsored by OpenAI. The app uses the real ChatGPT website; guest access and service availability can change.

## Requirements

- Windows 10/11 x64. Other architectures are not packaged by this workflow.
- Internet access for ChatGPT and, if needed, installation/update of WebView2 Evergreen.
- No Rust, Node.js, Docker, Visual Studio or developer tools are required by end users.

## Install

1. Download `ChatGPT-Island-Windows-x64-Setup.exe` and `SHA256SUMS.txt` from the **same published GitHub Release** at https://github.com/iiMedR/chatgpt-island/releases.
2. Compare `Get-FileHash .\ChatGPT-Island-Windows-x64-Setup.exe -Algorithm SHA256` with the checksum file before running it.
3. Exit any earlier Island process and close its conversation. Running two instances can conflict over Alt+Shift+E.
4. Run the installer. It installs for the current user and creates Start menu/uninstall entries. Administrator rights are not requested for the app installation itself; runtime installation or organizational policy may require additional approval.
5. Start **ChatGPT Island** from the Start menu. Normal startup shows a tray icon, without a launcher or empty window. Check hidden tray icons if necessary.

Current builds are unsigned. Windows may show an unknown-publisher/SmartScreen warning; do not treat a checksum as a substitute for trusting the source. Maintainer-controlled signing is not configured. The attached release manifest reports signature status.

## WebView2

The NSIS package embeds Microsoft's bootstrapper and checks a minimum runtime version of `110.0.1587.40`. If an adequate runtime is absent, the bootstrapper downloads/installs Evergreen. This is **not an offline runtime installer**. Network/proxy restrictions can prevent installation. Keep Evergreen updated; the app also verifies actual private mode and refuses a popup when that check fails. Runtime-floor configuration is not evidence that every older eligible runtime has been tested; native testing used the development host's newer runtime.

## Use

Select text in a supported application, press **Alt+Shift+E**, wait for the private ChatGPT draft, add a question and press Send yourself. With no selection, ChatGPT opens normally. The website may require login or verification; Island does not bypass it.

Escape in the popup, Alt+F4, or Alt+Shift+E requests closure. Cancel preserves the conversation. Confirm destroys the popup while the tray process stays alive. Settings can restore the close warning. Use tray Quit to request process exit; actual physical tray Quit delivery remains an outstanding runtime acceptance item.

Failed draft preparation opens no extra window. If needed, explicitly choose **Settings → Open manual handoff**, then select/copy/paste the draft yourself. Check the composer before pasting to avoid a duplicate. Manual Ctrl+C replaces your clipboard; the app does not do this automatically for handoff. Capture clipboard fallback is a different opt-in setting, disabled by default.

## Updates and uninstall

Exit Island before running a newer installer. Use the same per-user install scope and stable application identifier; do not manually mix standalone binaries with an installed running instance. Downgrades are disabled. There is no background updater or automatic startup registration.

Uninstall through **Windows Settings → Apps → Installed apps → ChatGPT Island → Uninstall**, or the installed uninstaller. Exit the tray process first. Installation, upgrade and uninstall behavior still require clean-machine acceptance; this document describes the configured route, not a claim those tests passed.

Settings are normally under `%APPDATA%\com.chatgptisland.dev`; fresh private-session directories are under `%LOCALAPPDATA%\com.chatgptisland.dev\private-sessions`. The legacy identifier is intentionally retained for compatibility. Do not remove profile files while Island or its app-owned WebView2 processes are using them. App-controlled automatic profile cleanup is not implemented. Inspect any uninstaller data-removal choice carefully; do not assume uninstall guarantees forensic erasure or deletion of remote ChatGPT data. Do not remove shared WebView2 solely because this app is uninstalled.

## Privacy

Island keeps one pending selection in memory and does not intentionally store selections/drafts in its settings, logs or analytics. Private profiles are not reused, but residual runtime files may remain. Putting a draft in ChatGPT exposes it to that page's scripts before Send; website-side draft processing and remote retention are outside Island's control. No automatic message submission occurs.
