# Landing page design

Approved concept: `../assets/design/landing-page-concept.png` (973 × 1616).

The supplied real demo video now replaces the original static selection-to-draft illustration, as requested. It uses native controls, a preview frame, and no autoplay or initial video preload. The implementation otherwise preserves the off-white palette, palm branding, centered three-line headline, primary black download button, shortcut keycaps, selection-to-draft illustration, three numbered steps, feature stripe, and independence disclosure. Text and controls are native Angular/HTML; the concept image is a design reference and is not loaded by the site.

The main components are `App` (page and privacy dialog), `DownloadButton` (direct installer link), `Icon`, and release URL constants (installer and checksum). OnPush keeps rendering simple. No router, backend, ChatGPT embedding, or desktop Tauri bridge is needed.

Reference comparison: header alignment and navigation hierarchy retained; hero copy and three-line composition retained; supplied logo retained; light document/blue highlight/dark draft relationship retained; step order and footer copy retained. Native fonts and spacing differ slightly from the raster concept. On narrow screens the preview and steps stack, the primary actions stack, and header links condense while equivalent links remain in the page/footer. Privacy opens in a keyboard-accessible native dialog; the installer and checksum are direct GitHub links.

The only intentional copy addition to the concept is an accessible explanation that the preview is an illustration. Interaction messages and privacy details supply information absent from the static image. No invented version, release availability, user metrics, testimonials, or affiliation claims are shown.
