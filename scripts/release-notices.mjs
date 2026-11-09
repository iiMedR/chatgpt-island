import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {readFileSync,readdirSync,existsSync,writeFileSync,mkdirSync} from 'node:fs';
import {dirname,join,basename,resolve} from 'node:path';
// Only license/notice files from resolved packages; never include source trees or local paths.
const metadata=JSON.parse(execFileSync('cargo',['metadata','--locked','--format-version','1','--filter-platform','x86_64-pc-windows-msvc'],{cwd:'src-tauri',encoding:'utf8',maxBuffer:32*1024*1024}));
const sections=['ChatGPT Island — third-party notices','The project itself is MIT licensed. Dependencies retain their own licenses.','This inventory includes resolved build-time packages as well as runtime packages.'];
const missing=[];
const supplemental=JSON.parse(readFileSync('assets/third-party/sources.json','utf8'));
function add(name,version,license,root,licenseFile,sourceUrl){
  const candidates=readdirSync(root,{withFileTypes:true}).filter(e=>e.isFile() && /^(licen[sc]e|copying|copyright|notice)([._-]|$)/i.test(e.name)).map(e=>join(root,e.name));
  if(licenseFile && existsSync(resolve(root,licenseFile))) candidates.push(resolve(root,licenseFile));
  const extra=supplemental.find(entry=>entry.name===name && entry.version===version);
  if(!candidates.length && extra) {
    const file=join('assets/third-party',extra.file);
    if(createHash('sha256').update(readFileSync(file)).digest('hex')!==extra.sha256) throw new Error('Supplemental license checksum mismatch: '+name);
    candidates.push(file);
  }
  const files=[...new Set(candidates)].filter(p=>!p.endsWith('.json'));
  sections.push(`\n${'='.repeat(72)}\n${name} ${version}\nDeclared license: ${license??'UNSPECIFIED'}`);
  sections.push('Unmodified source: '+(sourceUrl??('https://crates.io/crates/'+name+'/'+version)));
  if(extra) sections.push('Supplemental license source: '+extra.source);
  for(const file of files) sections.push(`\n--- ${basename(file)} ---\n${readFileSync(file,'utf8')}`);
  if(!files.length) { missing.push(`${name}@${version}`); sections.push('No standalone license file was found in the installed package; review the declared license before distribution.'); }
}
for(const p of metadata.packages.filter(p=>p.source).sort((a,b)=>(a.name+a.version).localeCompare(b.name+b.version))) add(p.name,p.version,p.license,dirname(p.manifest_path),p.license_file);
const lock=JSON.parse(readFileSync('package-lock.json','utf8'));
for(const [folder,p] of Object.entries(lock.packages).filter(([folder,p])=>folder && !p.dev && existsSync(join(folder,'package.json')))) {
 const pkg=JSON.parse(readFileSync(join(folder,'package.json'),'utf8')); add(pkg.name,p.version,pkg.license,folder,undefined,p.resolved);
}
writeFileSync('src-tauri/THIRD_PARTY_NOTICES.txt',sections.join('\n')+'\n');
mkdirSync('artifacts',{recursive:true});
writeFileSync('artifacts/license-audit.json',JSON.stringify({missingLicenseFiles:missing},null,2)+'\n');
console.log(`Generated dependency notices; ${missing.length} packages require manual license-file review.`);

if(missing.length) throw new Error('Dependency license files need review: '+missing.join(', '));
