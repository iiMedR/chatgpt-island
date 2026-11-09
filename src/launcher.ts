import { invoke } from "@tauri-apps/api/core";
export function showLauncher() {

document.querySelector<HTMLDivElement>("#app")!.innerHTML = `
  <h1>ChatGPT Island</h1>
  <p>Milestone 02 · development-only launcher</p>
  <p>This opens the real ChatGPT website in a private WebView2 window.</p>
  <button id="open">Open private popup</button>
  <button id="close">Request close</button>
  <p>For a fresh session: confirm close, wait for Closed, then open again.</p>
  <p id="state" aria-live="polite">Checking runtime…</p>
  <p id="message" role="status"></p>
  <p>Guest access depends on the website and network. If login or verification is required, stop the test. No bypass is provided.</p>
  <p>If the website is blank or offline, check your connection, destroy the popup and retry. Creation success does not prove navigation success.</p>
  <p>Profile directories are retained for safe offline cleanup. Closing does not delete remote service data.</p>`;
const message = document.querySelector<HTMLParagraphElement>("#message")!;
const state = document.querySelector<HTMLParagraphElement>("#state")!;
const buttons = [...document.querySelectorAll<HTMLButtonElement>("button")];
async function refresh() {
  try { state.textContent = `Popup: ${await invoke<string>("popup_status")}`; }
  catch (error) { state.textContent = `Runtime unavailable: ${String(error)}`; }
}
for (const [id, command] of [["open", "open_popup"], ["close", "close_popup"]]) {
  document.getElementById(id)!.addEventListener("click", async () => {
    buttons.forEach(button => button.disabled = true);
    try { message.textContent = await invoke<string>(command); }
    catch (error) { message.textContent = String(error); }
    finally { buttons.forEach(button => button.disabled = false); await refresh(); }
  });
}
void refresh();
const timer = window.setInterval(() => void refresh(), 1500);
window.addEventListener("pagehide", () => window.clearInterval(timer));

}
