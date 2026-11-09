# Production builds and release procedure

**Release readiness is NOT established by a successful build.** Installer execution/clean-machine upgrade/uninstall, unresolved tray Quit delivery and remaining runtime compatibility are acceptance gates. Hosted Actions execution is unverified until the workflow is explicitly pushed and run. Nothing in this work pushed commits, created tags/releases or uploaded binaries.

## Build design

- Tauri 2 NSIS, explicit `x86_64-pc-windows-msvc` target, optimized Rust release profile, per-user installation, LZMA compression, downgrades disabled.
- App identifier `com.chatgptisland.dev` retained to preserve existing state locations. Changing it later requires a migration decision.
- Final user-supplied logo converted with the Tauri CLI; `.ico` configured for executable, installer and uninstaller; default PNG used by the tray. No replacement artwork generated.
- WebView2 `embedBootstrapper`, silent runtime handling, minimum runtime `110.0.1587.40`. Internet is needed if the runtime must be installed. No full offline runtime is shipped.
- Explicit application resources: project LICENSE and generated THIRD_PARTY_NOTICES only. Frontend production assets are embedded; development launcher imports are tree-shaken out. No tests, source folders, private profiles, keys or signing certificates are configured as resources.
- Compiler paths are remapped in the release script to reduce disclosure of the local checkout/user-profile paths. This is not a claim that every upstream library is free of build metadata.
- Rust, Node, lockfiles and action SHAs constrain CI inputs. Windows runner images, NSIS/bootstrapper downloads and timestamps can still change; byte-for-byte reproducibility is not claimed.

Run from the repository root after installing development prerequisites:

```powershell
npm ci
.\scripts\verify.ps1
.\scripts\build-release.ps1
```

Docker frontend checks: `docker compose build` then `docker compose run --rm dev`. The native verification script checks TypeScript, JS/security tests, frontend build, Cargo check/tests, formatting and strict Clippy. Do not repeat full checks without a change/failure requiring it.

Build outputs:

```text
src-tauri/target/x86_64-pc-windows-msvc/release/chatgpt-island.exe
src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/*-setup.exe
artifacts/release/ChatGPT-Island-Windows-x64-Setup.exe
artifacts/release/SHA256SUMS.txt
artifacts/release/release-manifest.json
```

The release script rejects mismatched package versions, unexpected resource configuration and multiple stale installer outputs. It copies one installer to the stable asset name and calculates SHA-256. The manifest records version, architecture, size, hash, signature state and that execution acceptance has not been performed. Do not stage `artifacts/`, `target/`, local profiles or signing material.

Dependency notices are generated from resolved Cargo packages and production npm packages. Missing standalone licenses use vendored upstream texts with exact package-version/source/checksum records in `assets/third-party/sources.json`. A missing or modified supplemental license fails the build. MPL-covered unmodified source locations are listed; this inventory is not a legal opinion or a complete automated license-compatibility audit. Review obligations again when dependencies change. The repository's own code/assets are MIT licensed as selected by the owner; dependencies retain their licenses.

## GitHub Actions

`.github/workflows/release.yml` runs **only on pushed stable version tags**. Ordinary commits, PRs and manual dispatch cannot publish through this workflow. A tag such as `v0.1.0` must match package.json, Cargo.toml and tauri.conf.json. Prerelease tags are intentionally not supported yet.

1. Read-only `windows-2022` build job checks out the tag without persisting Git credentials, installs Node 24.22.1 and Rust 1.99.1, installs locked dependencies, verifies, builds NSIS and uploads only three allowlisted files as a workflow artifact.
2. A separate Ubuntu job downloads the same-run artifact, verifies its checksum and creates a **draft** GitHub Release containing the installer, checksum and manifest. Only that job has `contents: write`; it checks out/runs no repository code. All external actions are pinned to full commit SHAs. Dependabot proposes monthly action-pin updates.
3. The `release` environment should require maintainer review and allow only version tags. This requires repository-owner configuration; adding the workflow alone does not enable protection. Draft publication is a separate manual action after acceptance.

No PAT, signing certificate, updater key or other secret is needed for this unsigned draft-release route. `GITHUB_TOKEN` is provided only to the draft creation step. Do not grant write permissions to the build job or add `pull_request_target` execution of contributor code. Protect tags against deletion/retargeting. Rerunning draft creation for an existing tag fails safely instead of replacing published assets.

## Website download contract

Use the exact same asset names in every stable release:

```text
https://github.com/iiMedR/chatgpt-island/releases/latest/download/ChatGPT-Island-Windows-x64-Setup.exe
https://github.com/iiMedR/chatgpt-island/releases/latest/download/SHA256SUMS.txt
https://github.com/iiMedR/chatgpt-island/releases/latest/download/release-manifest.json
```

Version-pinned URLs replace `latest/download` with `download/v0.1.0`, for example:

```text
https://github.com/iiMedR/chatgpt-island/releases/download/v0.1.0/ChatGPT-Island-Windows-x64-Setup.exe
```

These URLs will not work until a corresponding public release with those assets is published. Drafts are not website downloads. Publishing the intended stable release as Latest is a maintainer action; do not point the website at transient Actions artifacts or raw Git blobs.

To display the current version, request:

```text
GET https://api.github.com/repos/iiMedR/chatgpt-island/releases/latest
```

Use `tag_name`, `published_at`, `html_url`, and `assets[].browser_download_url` matched by exact asset name. Handle 404 before the first public release, missing assets, rate limits and network errors. Cache responses server-side (for example 10 minutes), use ETag/If-None-Match when appropriate, and never embed a GitHub token in website JavaScript. The public endpoint needs no token for ordinary unauthenticated access. No website is built in this task.

## First release — owner actions, not executed here

1. Review local conventional commits and the package audit/build report. Resolve the release gates below before publication.
2. In GitHub, enable private vulnerability reporting, verify intended public visibility, create/protect the `release` environment and restrict version tag changes. Decide on Windows code signing; currently outputs are unsigned.
3. Update the Unreleased changelog into a dated 0.1.0 entry only when actually releasing. Keep all three version files aligned, update the corresponding npm/Cargo lockfiles, and regenerate dependency notices if dependencies change.
4. **Authorize the push first.** Then push the reviewed current branch with `git push origin master`. This work does not perform that action.
5. After explicit release/tag authorization, run:

   ```powershell
   git tag -a v0.1.0 -m "ChatGPT Island 0.1.0"
   git push origin v0.1.0
   ```

6. Inspect the tag workflow. Approve its protected environment if configured. It creates a draft, not a public release. Confirm exact files, sizes, SHA-256 and signature status. Hosted execution has not been tested locally.
7. Download the draft installer as maintainer, verify the checksum, and complete Windows acceptance below. Do not bypass security prompts or test against personal conversations/profiles.
8. Edit release notes to state actual tested platforms, known issues, unsigned status if applicable, and the independent-project notice. Only then manually publish the draft and mark the intended stable release as Latest.
9. Verify public stable/versioned URLs and API version metadata. The website can then link to the stable installer URL. No automatic app updater is configured.

## Acceptance gates

- [ ] Clean Windows x64 install with an existing supported WebView2 runtime.
- [ ] Runtime bootstrapper behavior with runtime absent/outdated, network/proxy failure and cancellation.
- [ ] Correct app/installer/uninstaller/Start menu/tray icon, including small-size visibility.
- [ ] Installed app startup, one-instance shortcut behavior and actual tray Quit/shortcut release.
- [ ] Selection → editable draft; no automatic Send; close/Cancel/reopen isolation.
- [ ] Upgrade from an older installed version; retained preferences; downgrade refusal.
- [ ] Uninstall removes program/shortcuts correctly; verify data-removal behavior and no orphan app-owned processes.
- [ ] Clean package resource/secret review and dependency notice obligations.
- [ ] Signing/reputation decision, or explicit acceptance of an unsigned release with warnings documented.
- [ ] Hosted tag workflow and public download/API behavior after owner-authorized publication.

Installation acceptance has not been silently performed against the developer's existing installation or private profiles. Keep a VM/Sandbox snapshot for first-install/upgrade/uninstall cases.

References: [Tauri Windows installers](https://v2.tauri.app/distribute/windows-installer/), [GitHub release links](https://docs.github.com/en/repositories/releasing-projects-on-github/linking-to-releases), [GitHub Releases API](https://docs.github.com/en/rest/releases/releases), [GitHub Actions security](https://docs.github.com/en/actions/reference/security/secure-use).
