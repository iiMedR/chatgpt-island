import { invoke } from "@tauri-apps/api/core";
interface Snapshot {
  phase: string;
  session_id: string | null;
  skip_close_confirmation: boolean;
  shortcut_registered: boolean;
  shortcut: string;
  error: string | null;
}
const root = () => document.querySelector<HTMLElement>("#app")!;
export async function showConfirmation() {
  document.title = "Close this conversation?";
  root().innerHTML = `<section class="dialog"><h1>Close this conversation?</h1>
    <p>This conversation is temporary and won't be available after you close the window.</p>
    <label class="check"><input type="checkbox" id="skip"> Don't show this again</label>
    <p id="error" role="alert"></p><div class="actions"><button id="cancel" autofocus>Cancel</button><button id="confirm" class="primary">Close conversation</button></div></section>`;
  const sessionId = new URLSearchParams(window.location.search).get("session");
  const error = document.querySelector<HTMLElement>("#error")!;
  const buttons = [...document.querySelectorAll<HTMLButtonElement>("button")];
  let pending = false;
  async function decide(confirm: boolean) {
    if (pending) return;
    pending = true; buttons.forEach(button => button.disabled = true);
    try {
      if (!sessionId) throw new Error("No pending session.");
      await invoke("resolve_confirmation", { sessionId, confirm, skip: document.querySelector<HTMLInputElement>("#skip")!.checked });
    } catch (e) { error.textContent = String(e); pending = false; buttons.forEach(button => button.disabled = false); }
  }
  document.querySelector<HTMLButtonElement>("#cancel")!.onclick = () => void decide(false);
  document.querySelector<HTMLButtonElement>("#confirm")!.onclick = () => void decide(true);
  document.addEventListener("keydown", event => { if (event.key === "Escape") { event.preventDefault(); void decide(false); } });
  document.querySelector<HTMLButtonElement>("#cancel")!.focus();
}
export async function showSettings() {
  document.title = "ChatGPT Island — Settings";
  root().innerHTML = `<h1>ChatGPT Island</h1><p>Settings</p><dl><dt>Global shortcut</dt><dd id="shortcut"></dd><dt>Close warning</dt><dd id="warning"></dd></dl>
    <button id="reset">Show close warning again</button><p id="status" role="status"></p><p id="error" role="alert"></p>`;
  const error = document.querySelector<HTMLElement>("#error")!;
  async function refresh() {
    const state = await invoke<Snapshot>("ui_snapshot");
    document.querySelector<HTMLElement>("#shortcut")!.textContent = `${state.shortcut} — ${state.shortcut_registered ? "registered" : "unavailable; use the tray menu"}`;
    document.querySelector<HTMLElement>("#warning")!.textContent = state.skip_close_confirmation ? "Disabled" : "Enabled";
    error.textContent = state.error ?? "";
  }
  const reset = document.querySelector<HTMLButtonElement>("#reset")!;
  reset.onclick = async () => {
    reset.disabled = true;
    try { await invoke("reset_warning"); await refresh(); document.querySelector<HTMLElement>("#status")!.textContent = "Close warning enabled."; }
    catch (e) { error.textContent = String(e); }
    finally { reset.disabled = false; }
  };
  try { await refresh(); } catch (e) { error.textContent = String(e); }
}
