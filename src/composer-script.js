// Executed once per bounded native request, never installed as a page listener.
(payload => {
  try {
    if (window !== window.top || location.origin !== 'https://chatgpt.com' ||
        location.pathname !== '/' || location.search || location.hash) return 'blocked';
    if (document.readyState === 'loading') return 'waiting';
    if (document.querySelector('input[type="password"], #challenge-running, iframe[src*="challenges.cloudflare.com"]')) return 'blocked';
    const candidates = [...document.querySelectorAll('#prompt-textarea, textarea[name="prompt-textarea"], #mobile-composer-prompt')]
      .filter(e => (e instanceof HTMLTextAreaElement || e.isContentEditable) &&
        !e.disabled && !e.readOnly && e.getAttribute('aria-disabled') !== 'true' &&
        e.getBoundingClientRect().width > 0 && e.getBoundingClientRect().height > 0 &&
        getComputedStyle(e).visibility === 'visible');
    if (candidates.length !== 1) return 'waiting';
    const editor = candidates[0];
    const value = () => (editor instanceof HTMLTextAreaElement ? editor.value : editor.innerText).replace(/\r\n/g, '\n');
    if (payload.mode === 'probe') return value().trim() ? 'occupied' : 'ready';
    if (payload.mode === 'verify') {
      if (value().trimEnd() !== payload.draft.trimEnd()) return 'changed';
      const send = document.querySelector('button[data-testid="send-button"], button#composer-submit-button, button[aria-label="Send prompt"], button[aria-label="Send message"]');
      return send && !send.disabled && send.getAttribute('aria-disabled') !== 'true' ? 'accepted' : 'unavailable';
    }
    if (payload.mode !== 'insert' || value().trim()) return 'occupied';
    // No HTML, synthetic Enter, form submission, send click, storage or network APIs.
    editor.focus();
    if (document.activeElement !== editor) return 'unavailable';
    if (editor instanceof HTMLTextAreaElement) {
      const setter = Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, 'value')?.set;
      if (!setter) return 'unavailable';
      setter.call(editor, payload.draft);
      editor.dispatchEvent(new InputEvent('input', { bubbles: true, inputType: 'insertText', data: payload.draft }));
      editor.setSelectionRange(editor.value.length, editor.value.length);
    } else {
      // Browser editing keeps contenteditable editor state/undo, unlike textContent assignment.
      const selection = getSelection();
      const range = document.createRange();
      range.selectNodeContents(editor); range.collapse(false);
      selection.removeAllRanges(); selection.addRange(range);
      if (!document.execCommand('insertText', false, payload.draft)) return 'unavailable';
    }
    return value().trimEnd() === payload.draft.trimEnd() ? 'inserted' : 'unavailable';
  } catch { return 'unavailable'; }
})(__ISLAND_PAYLOAD__)
