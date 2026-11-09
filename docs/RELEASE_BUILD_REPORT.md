# Local release build report — 2026-11-09

## Status

**Infrastructure implemented; NOT release-ready.** No installer was executed against the developer's active application or profiles. Clean-machine installation/upgrade/uninstall, WebView2 bootstrapper execution, hosted Actions execution and remaining runtime acceptance are unverified. No commits were pushed, no tags/releases were created and no binaries were uploaded.

## Actual outputs

| Artifact | Repository-relative path | Size |
|---|---|---:|
| Stable NSIS installer | `artifacts/release/ChatGPT-Island-Windows-x64-Setup.exe` | 3,327,143 bytes (3.17 MiB) |
| Original NSIS output | `src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/ChatGPT Island_0.1.0_x64-setup.exe` | 3,327,143 bytes |
| Application executable | `src-tauri/target/x86_64-pc-windows-msvc/release/chatgpt-island.exe` | 4,632,064 bytes |
| Checksum file | `artifacts/release/SHA256SUMS.txt` | Generated for the stable installer filename |
| Release metadata | `artifacts/release/release-manifest.json` | Version, architecture, bytes, hash, signature/test status |

Installer SHA-256:

```text
4bd28f986dafea06d667648b058fa3ab4a725ccbc1c5356fad1b905bc863e394
```

Application SHA-256:

```text
2aa11819c4c49785bdaeef7bf31c2fe2c131d21064caee6d0694348fbfd32da4
```

Version information reports **ChatGPT Island 0.1.0**. App payload is Windows x64 (`0x8664`), GUI subsystem 2. NSIS uses a 32-bit setup engine (`0x14c`) to install the x64 payload; that stub architecture is expected. The app's import table contained Windows system/UCRT DLLs and no direct VCRUNTIME/MSVCP redistributable DLL imports. This does not replace clean-machine testing.

## Verification performed

| Check | Actual result |
|---|---|
| Docker image build and `docker compose run --rm dev` | PASS: TypeScript, 4 JS/security tests, frontend production build |
| `.\scripts\verify.ps1` | PASS: TypeScript, 4 JS tests, frontend build, Cargo check, 30 Rust tests, formatting, strict Clippy |
| `.\scripts\build-release.ps1` | PASS: optimized explicit x64 application, NSIS setup, stable copy, SHA-256 and manifest |
| Production frontend preflight | PASS: development launcher omitted; no source maps; exact local capability roles; no remote capabilities |
| Version/resource preflight | PASS: 0.1.0 aligned; NSIS/current-user/embedded bootstrapper; only LICENSE/notices resources |
| Dependency notice generation | PASS: zero missing license-file entries after vendoring upstream texts; per-package source/license inventory produced |
| Final logo | PASS: supplied replacement preserved through Tauri conversion; PNG transparency retained; ICO frames 16/24/32/48/64/256; generated 128px image visually inspected |
| Binary/resource secret-pattern and build-user-path checks | No matching credential patterns or local user-profile paths found in checked app/setup/resource bytes |
| Generated NSIS File input review | Only app executable, LICENSE, THIRD_PARTY_NOTICES and Microsoft bootstrapper as active app payload inputs; no source/test/profile/key directories configured |
| Embedded Microsoft bootstrapper signature | Valid; Microsoft Corporation signer |
| Our installer signature | NotSigned; no certificate provisioned |
| Workflow policy/static review | Four immutable official Action pins, tag-only trigger, read-only build job, separate write-capable draft-release job; hosted execution NOT TESTED |
| Documentation links | Local documentation links checked |

Tauri downloaded and hash-validated NSIS 3.13 and nsis-tauri-utils 0.5.3. Microsoft bootstrapper was obtained from the official download endpoint. Build compiler paths were remapped. Raw-byte secret-pattern checks and generated-input review are limited checks, not proof against every possible sensitive value or a full security audit. The compressed installer archive was not independently decompressed. Installer, upgrade and uninstaller were not executed.

The generated uninstaller script offers an explicit app-data deletion checkbox; when selected outside update mode it removes the app's APPDATA and LOCALAPPDATA identifier directories. Users must exit the app and release its WebView2 processes before choosing deletion. The application itself still does not automatically remove active/private profiles. Actual data-removal behavior remains an installation acceptance item.

## Open-source and publishing checklist

- [x] Owner selected MIT; project LICENSE and manifest metadata prepared.
- [x] Final owner-supplied logo configured for native app/tray/installer/uninstaller.
- [x] CONTRIBUTING, SECURITY, CHANGELOG, README, installation and release docs prepared.
- [x] Independent/non-affiliation notice included.
- [x] Build artifacts, private profiles and signing material ignored; external resources allowlisted.
- [x] Dependency notices and missing-file checks added; dependency licenses retain their own terms.
- [x] Stable installer/checksum/manifest names and latest/versioned download/API strategy documented.
- [ ] Maintainer review of dependency obligations and the package audit scope.
- [ ] Clean Windows install, runtime acquisition/failure, upgrade, uninstall and icon cache checks.
- [ ] Actual tray Quit/shortcut release and remaining functional/platform acceptance.
- [ ] Signing decision/certificate provisioning; current artifacts unsigned.
- [ ] Repository private vulnerability reporting, protected tags and release-environment reviewers configured by owner.
- [ ] Owner-authorized push/tag; hosted workflow run; draft asset review; explicit public publication.

Follow [RELEASING.md](RELEASING.md) for exact commands and publication sequence. The repository currently contains infrastructure rather than a public release announcement. No website was built and no automatic updater was added.
