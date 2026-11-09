import assert from 'node:assert/strict';
import test from 'node:test';
import vm from 'node:vm';
import {readFileSync} from 'node:fs';
const source = readFileSync(new URL('../src/composer-script.js', import.meta.url), 'utf8');
function fixture({origin='https://chatgpt.com', path='/', value='', missing=false, protectedInput=false}={}) {
  class Textarea {
    #value = value;
    get value(){return this.#value} set value(v){this.#value=v}
    getAttribute(){return null} getBoundingClientRect(){return {width:100,height:40}}
    focus(){document.activeElement=this} dispatchEvent(){this.inputs=(this.inputs??0)+1} setSelectionRange(){}
  }
  const editor = new Textarea();
  const send = {disabled:false,getAttribute(){return null}};
  const document = {readyState:'complete',activeElement:null,
    querySelector(q){return q.includes('password') ? (protectedInput ? {} : null) : send},
    querySelectorAll(){return missing ? [] : [editor]}};
  const window = {}; window.top = window;
  const context = {window,location:{origin,pathname:path,search:'',hash:''},document,HTMLTextAreaElement:Textarea,
    InputEvent:class {},getComputedStyle:()=>({visibility:'visible'})};
  const run = (mode,draft='') => vm.runInNewContext(source.replace('__ISLAND_PAYLOAD__',JSON.stringify({mode,draft})),context);
  return {editor,run};
}
test('composer waits, rejects unexpected/protected pages and preserves existing user drafts',()=>{
  assert.equal(fixture({missing:true}).run('probe'),'waiting');
  assert.equal(fixture({origin:'https://evil.test'}).run('insert','secret'),'blocked');
  assert.equal(fixture({path:'/auth/login'}).run('insert','secret'),'blocked');
  assert.equal(fixture({protectedInput:true}).run('probe'),'blocked');
  const f=fixture({value:'my existing draft'}); assert.equal(f.run('insert','new'),'occupied'); assert.equal(f.editor.value,'my existing draft');
});
test('draft stays editable data and a duplicate insertion cannot overwrite it',()=>{
  const f=fixture(); const draft='Français\nمرحبا\n"; throw new Error("executed"); //';
  assert.equal(f.run('probe'),'ready'); assert.equal(f.run('insert',draft),'inserted');
  assert.equal(f.editor.value,draft); assert.equal(f.editor.inputs,1);
  assert.equal(f.run('verify',draft),'accepted'); assert.equal(f.run('insert',draft),'occupied');
  f.editor.value+=' My question'; assert.equal(f.run('verify',draft),'changed');
});
