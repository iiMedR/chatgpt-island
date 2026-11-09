# Brand assets

`logo.png` is the final replacement logo supplied by the project owner for this release work. No replacement design was generated. `npm run tauri -- icon assets/branding/logo.png --output artifacts/icon-build` performs format/size conversion; root-level generated icons are copied into `src-tauri/icons/`.

The app/tray use the configured default PNG icon; Windows executable/NSIS installer/uninstaller use `icon.ico`. The original pixels and transparency are preserved apart from resampling by the Tauri converter. Small-size legibility and Explorer/Start menu icon caches should be checked during installation acceptance.

ChatGPT Island is independent of OpenAI. The supplied island mark is application branding; it is not an OpenAI endorsement.
