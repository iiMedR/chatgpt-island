import {readFileSync,readdirSync,existsSync} from 'node:fs';
import assert from 'node:assert/strict';
const read=p=>readFileSync(p,'utf8');
const pkg=JSON.parse(read('package.json')), config=JSON.parse(read('src-tauri/tauri.conf.json'));
const cargo=read('src-tauri/Cargo.toml').match(/^version = "([^"]+)"/m)?.[1];
assert.equal(pkg.version,config.version);assert.equal(cargo,config.version);
const lock=JSON.parse(read('package-lock.json'));assert.equal(lock.version,pkg.version);assert.equal(lock.packages[''].version,pkg.version);
if(process.env.RELEASE_TAG) assert.equal(process.env.RELEASE_TAG,`v${config.version}`,'Tag must match all package versions');
assert.equal(pkg.license,'MIT');assert.ok(existsSync('LICENSE'));
assert.deepEqual(config.bundle.targets,['nsis']);assert.equal(config.bundle.active,true);
assert.deepEqual(Object.keys(config.bundle.resources).sort(),['../LICENSE','THIRD_PARTY_NOTICES.txt']);
assert.equal(config.bundle.windows.nsis.installMode,'currentUser');
assert.equal(config.bundle.windows.webviewInstallMode.type,'embedBootstrapper');
assert.equal(config.app.withGlobalTauri,false);assert.deepEqual(config.app.windows,[]);
for(const name of readdirSync('src-tauri/capabilities')) { const cap=JSON.parse(read(`src-tauri/capabilities/${name}`)); assert.equal(cap.remote,undefined);assert.equal(cap.local,true);assert.ok(cap.windows.every(w=>!w.includes('*')));assert.ok(cap.permissions.every(p=>!p.includes(':')&&!p.includes('*'))); }
if(process.argv.includes('--frontend') && existsSync('dist/assets')) for(const name of readdirSync('dist/assets').filter(n=>n.endsWith('.js'))) {
 const content=read(`dist/assets/${name}`);
 assert.ok(!content.includes('development-only launcher'),'Development launcher shipped in production');
 assert.ok(!/sk-(?:proj-)?[A-Za-z0-9_-]{20,}|gh[pousr]_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{30,}|AKIA[0-9A-Z]{16}|-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----/.test(content),'Potential secret in frontend');
}
console.log(`Release configuration and frontend boundaries checked for v${config.version}.`);
