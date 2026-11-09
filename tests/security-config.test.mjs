import { readFileSync, readdirSync } from 'node:fs';
import assert from 'node:assert/strict';
import test from 'node:test';
const read = path => readFileSync(new URL(`../${path}`, import.meta.url), 'utf8');
const config = JSON.parse(read('src-tauri/tauri.conf.json'));
test('capabilities separate local UI roles and grant no remote/native plugin access', () => {
  const expected = {
    'handoff.json': { windows: ['handoff'], permissions: ['allow-handoff-focus', 'allow-handoff-snapshot'] },
    'default.json': { windows: ['main'], permissions: ['allow-close-popup', 'allow-open-popup', 'allow-popup-status', 'allow-selection-preview'] },
    'confirmation.json': { windows: ['confirmation'], permissions: ['allow-resolve-confirmation'] },
    'settings.json': { windows: ['settings'], permissions: ['allow-reset-warning', 'allow-set-clipboard-fallback', 'allow-ui-snapshot'] },
  };
  assert.deepEqual(readdirSync(new URL('../src-tauri/capabilities/', import.meta.url)).sort(), Object.keys(expected).sort());
  for (const [file, role] of Object.entries(expected)) {
    const capability = JSON.parse(read(`src-tauri/capabilities/${file}`));
    assert.deepEqual(capability.windows, role.windows);
    assert.equal(capability.local, true);
    assert.equal(capability.remote, undefined);
    assert.deepEqual(capability.permissions.sort(), role.permissions);
    assert.ok(capability.permissions.every(permission => !permission.includes(':') && !permission.includes('*')));
  }
  assert.equal(config.app.withGlobalTauri, false);
  assert.equal(config.app.security.dangerousDisableAssetCspModification, undefined);
  assert.ok(config.app.security.csp.includes("default-src 'self'"));
});
test('normal startup creates no windows and development remains loopback-only', () => {
  assert.equal(config.build.devUrl, 'http://127.0.0.1:1420');
  assert.equal(config.bundle.active, false);
  assert.deepEqual(config.app.windows, []);
  assert.match(read('vite.config.ts'), /host: "127\.0\.0\.1"/);
});
