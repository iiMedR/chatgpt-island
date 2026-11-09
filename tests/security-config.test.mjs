import { readFileSync, readdirSync } from 'node:fs';
import assert from 'node:assert/strict';
import test from 'node:test';
const read = path => readFileSync(new URL(`../${path}`, import.meta.url), 'utf8');
const config = JSON.parse(read('src-tauri/tauri.conf.json'));
test('configuration confines capabilities to the local launcher', () => {
  const files = readdirSync(new URL('../src-tauri/capabilities/', import.meta.url));
  assert.deepEqual(files, ['default.json']);
  const capability = JSON.parse(read('src-tauri/capabilities/default.json'));
  assert.deepEqual(capability.windows, ['main']);
  assert.equal(capability.local, true);
  assert.equal(capability.remote, undefined);
  assert.deepEqual(capability.permissions.sort(), ['allow-close-popup', 'allow-open-popup', 'allow-popup-status']);
  assert.equal(config.app.withGlobalTauri, false);
  assert.equal(config.app.security.dangerousDisableAssetCspModification, undefined);
  assert.ok(config.app.security.csp.includes("default-src 'self'"));
});
test('development is loopback-only and installers are disabled', () => {
  assert.equal(config.build.devUrl, 'http://127.0.0.1:1420');
  assert.equal(config.bundle.active, false);
  assert.deepEqual(config.app.windows.map(window => window.label), ['main']);
  assert.match(read('vite.config.ts'), /host: "127\.0\.0\.1"/);
});
