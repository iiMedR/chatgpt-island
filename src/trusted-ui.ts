import { invoke } from "@tauri-apps/api/core";
interface Snapshot {
  clipboard_fallback_enabled: boolean;
  capture_status: string;
  phase: string;
  session_id: string | null;
  skip_close_confirmation: boolean;
  shortcut_registered: boolean;
  shortcut: string;
  error: string | null;
}
const root = () => document.querySelector<HTMLElement>("#app")!;
export async function showHandoff() {
  document.title = "Selected text — manual handoff";
  root().innerHTML = `<h1>Paste your selected text</h1><p>Automatic preparation could not be confirmed. Check the ChatGPT composer first: a draft may already be present.</p>
    <p>Select the draft below, press Ctrl+C, return to ChatGPT and paste once with Ctrl+V. Copying replaces your clipboard contents. Nothing is submitted for you.</p>
    <textarea id="draft" readonly aria-label="Prepared draft"></textarea>
    <button id="select">Select draft</button><button id="focus">Return to ChatGPT</button><p id="error" role="alert"></p>`;
  const sessionId = new URLSearchParams(window.location.search).get("session");
  const text = document.querySelector<HTMLTextAreaElement>("#draft")!;
  const error = document.querySelector<HTMLElement>("#error")!;
  try { const state = await invoke<{draft: string}>("handoff_snapshot", {sessionId}); text.value = state.draft; }
  catch { error.textContent = "This selection is no longer available."; }
  document.querySelector<HTMLButtonElement>("#select")!.onclick = () => { text.focus(); text.select(); };
  document.querySelector<HTMLButtonElement>("#focus")!.onclick = async () => {
    try { await invoke("handoff_focus", {sessionId}); } catch { error.textContent = "This popup has closed."; text.value = ""; }
  };
  window.addEventListener("pagehide", () => { text.value = ""; });
}
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
    <button id="reset">Show close warning again</button>
    <h2>Selected text</h2><label class="check"><input type="checkbox" id="clipboard"> Allow clipboard fallback</label>
    <p>Off by default. If accessibility identifies a selection but cannot read it, this sends Ctrl+C and temporarily reads the copy. It requires an empty clipboard to protect existing formats. Clipboard managers and delayed copies may prevent restoration.</p>
    <p id="capture" role="status"></p><p id="status" role="status"></p><p id="error" role="alert"></p>`;
  const error = document.querySelector<HTMLElement>("#error")!;
  async function refresh() {
    const state = await invoke<Snapshot>("ui_snapshot");
    document.querySelector<HTMLElement>("#shortcut")!.textContent = `${state.shortcut} — ${state.shortcut_registered ? "registered" : "unavailable; use the tray menu"}`;
    document.querySelector<HTMLElement>("#warning")!.textContent = state.skip_close_confirmation ? "Disabled" : "Enabled";
    error.textContent = state.error ?? "";
    document.querySelector<HTMLInputElement>("#clipboard")!.checked = state.clipboard_fallback_enabled;
    document.querySelector<HTMLElement>("#capture")!.textContent = state.capture_status;
  }
  const clipboard = document.querySelector<HTMLInputElement>("#clipboard")!;
  clipboard.onchange = async () => {
    clipboard.disabled = true;
    try { await invoke("set_clipboard_fallback", { enabled: clipboard.checked }); await refresh(); }
    catch (e) { error.textContent = String(e); }
    finally { clipboard.disabled = false; }
  };
  const reset = document.querySelector<HTMLButtonElement>("#reset")!;
  reset.onclick = async () => {
    reset.disabled = true;
    try { await invoke("reset_warning"); await refresh(); document.querySelector<HTMLElement>("#status")!.textContent = "Close warning enabled."; }
    catch (e) { error.textContent = String(e); }
    finally { reset.disabled = false; }
  };
  try { await refresh(); } catch (e) { error.textContent = String(e); }
}
