# Angular landing page

Standalone Angular 22 application. It is separate from the Tauri frontend; commands here do not build or modify the desktop application.

## Local development

Use Node 24.22.1 (or an Angular-supported Node version) and npm:

```powershell
cd landing-page
npm ci
npm start
```

Open the local URL printed by Angular. Production output:

```powershell
npm run build
```

Publish only `dist/landing-page/browser/`.

## Cloudflare Pages

Connect the repository, choose `master` as the production branch, set root directory to `landing-page`, build command to `npm ci && npm run build`, and output directory to `dist/landing-page/browser`. The `.node-version` pins Node. No secrets or environment variables are required. Deploying a connected branch publishes it: configure this only when ready. No deployment has been performed by adding these files.

The `_headers` file sets a CSP and privacy/security headers on Cloudflare Pages. Angular inserts component styles at runtime, requiring inline styles; inline JavaScript remains blocked. No backend or Pages Function is needed.

## Downloads

Installers live in GitHub Releases, never in this site. Header and hero download links point directly to the supplied installer, without an API request or API rate-limit dependency:

- Installer: https://github.com/iiMedR/chatgpt-island/releases/download/v0.1.0/ChatGPT-Island-Windows-x64-Setup.exe
- Checksum: https://github.com/iiMedR/chatgpt-island/releases/download/v0.1.0/SHA256SUMS.txt

The v0.1.0 release is public. These versioned links stay pinned to that release; update `src/app/release-download.ts` when promoting a new version.

Future releases can use GitHub's stable alias after a regular release is published:
`https://github.com/iiMedR/chatgpt-island/releases/latest/download/ChatGPT-Island-Windows-x64-Setup.exe`

## Design and privacy

The approved reference is `../assets/design/landing-page-concept.png`. The embedded demo is a local prerecorded MP4 with native playback controls, a poster, muted autoplay, looping, and `preload="auto"`. No remote iframe or live composer is loaded. The original supplied video remains untouched on the Desktop. The web copy preserves 1752 × 1080 resolution, roughly 60 fps, and the original audio; H.264 CRF 18 compression reduced it from 21,395,070 to 5,458,483 bytes (74.5%). Full-video SSIM against the original: 0.999164. This is high-quality lossy compression, not mathematically lossless. The demo has no audible narration; the page describes the workflow in text. The supplied palm logo is preserved. Inter is self-hosted under its bundled SIL Open Font License (`public/assets/fonts/OFL.txt`). Application source uses the repository MIT license.

No analytics, tracking cookies, external font service, or API credentials are included. The privacy dialog explains draft exposure and residual profile files. ChatGPT Island is independent and not affiliated with OpenAI.
