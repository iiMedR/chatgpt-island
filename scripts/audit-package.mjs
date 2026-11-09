import {readFileSync,readdirSync,writeFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
const base='src-tauri/target/x86_64-pc-windows-msvc/release';
const app=`${base}/chatgpt-island.exe`, setup='artifacts/release/ChatGPT-Island-Windows-x64-Setup.exe';
function inspect(file) {
 const b=readFileSync(file), pe=b.readUInt32LE(0x3c);
 assert.equal(b.toString('ascii',0,2),'MZ');assert.equal(b.toString('ascii',pe,pe+4),'PE\0\0');
 const machine=b.readUInt16LE(pe+4),opt=pe+24,sectionCount=b.readUInt16LE(pe+6),sectionStart=opt+b.readUInt16LE(pe+20),sections=[];
 for(let i=0;i<sectionCount;i++){const at=sectionStart+i*40;sections.push({va:b.readUInt32LE(at+12),size:Math.max(b.readUInt32LE(at+8),b.readUInt32LE(at+16)),raw:b.readUInt32LE(at+20)});}
 const offset=rva=>{const s=sections.find(s=>rva>=s.va&&rva<s.va+s.size);return s?s.raw+rva-s.va:rva;};
 const directory=opt+(b.readUInt16LE(opt)===0x20b?112:96),imports=[];
 let at=offset(b.readUInt32LE(directory+8));
 for(let i=0;i<256 && at && b.readUInt32LE(at+12);i++,at+=20){const start=offset(b.readUInt32LE(at+12));const end=b.indexOf(0,start);imports.push(b.toString('ascii',start,end));}
 const secret=/sk-(?:proj-)?[A-Za-z0-9_-]{20,}|gh[pousr]_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{30,}|AKIA[0-9A-Z]{16}|-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----/;
 for(const encoding of ['latin1','utf16le']){
   const text=b.toString(encoding);assert.ok(!secret.test(text),'Potential credential pattern in '+file);
   if(process.env.USERPROFILE) for(const prefix of [process.env.USERPROFILE,process.env.USERPROFILE.replaceAll('\\','/')]) assert.ok(!text.toLowerCase().includes(prefix.toLowerCase()),'Build-machine user path in '+file);
 }
 return {file:file.split('/').pop(),bytes:b.length,sha256:createHash('sha256').update(b).digest('hex'),machine:`0x${machine.toString(16)}`,subsystem:b.readUInt16LE(opt+68),imports};
}
const executable=inspect(app),installer=inspect(setup);
assert.equal(executable.machine,'0x8664','Application payload must be x64');assert.equal(executable.subsystem,2,'Release app must be Windows GUI subsystem');
assert.ok(!executable.imports.some(n=>/^(vcruntime|msvcp)\d/i.test(n)),'Unexpected VC++ redistributable dependency; review installer handling');
const nsi=readFileSync(`${base}/nsis/x64/installer.nsi`,'utf8');
const fileLines=nsi.split(/\r?\n/).filter(line=>/^\s*File\s/i.test(line));
for(const line of fileLines) assert.ok(/MAINBINARYSRCPATH|WEBVIEW2BOOTSTRAPPERPATH|WEBVIEW2INSTALLERPATH|oname=LICENSE|oname=THIRD_PARTY_NOTICES\.txt/.test(line),'Unexpected NSIS file input');
assert.equal(fileLines.length,5,'Review changed NSIS input template');
assert.ok(nsi.includes('com.chatgptisland.dev'));assert.ok(nsi.includes('embedBootstrapper'));
const checksums=readFileSync('artifacts/release/SHA256SUMS.txt','utf8').trim().split(/\s+/);assert.equal(checksums[0],installer.sha256);assert.equal(checksums[1],installer.file);
assert.ok(readdirSync('dist/assets').every(n=>!n.endsWith('.map')),'Source maps found');
for(const file of ['LICENSE','src-tauri/THIRD_PARTY_NOTICES.txt']) {
 const content=readFileSync(file,'utf8');
 assert.ok(!/sk-(?:proj-)?[A-Za-z0-9_-]{20,}|gh[pousr]_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{30,}|AKIA[0-9A-Z]{16}|-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----/.test(content),'Potential credential pattern in resource '+file);
 if(process.env.USERPROFILE) assert.ok(!content.toLowerCase().includes(process.env.USERPROFILE.toLowerCase()),'Local profile path in resource');
}
const result={executable,installer,nsisApplicationInputs:['application executable','LICENSE','THIRD_PARTY_NOTICES.txt','Microsoft WebView2 bootstrapper'],potentialCredentialPatternsFound:false,buildMachineUserPathsFound:false,remoteCapabilities:'none (configuration preflight checked)',scope:'PE/import/byte-pattern checks, production frontend and generated NSIS input allowlist; archive not independently decompressed; installer not executed'};
writeFileSync('artifacts/package-audit.json',JSON.stringify(result,null,2)+'\n');console.log(JSON.stringify(result,null,2));
