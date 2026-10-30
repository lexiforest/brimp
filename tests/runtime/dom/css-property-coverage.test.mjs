import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
import {createSession} from '../../../bindings/node/index.js';

const inventory=JSON.parse(fs.readFileSync(new URL('../fixtures/environment-inventory.safari.json',import.meta.url),'utf8'));
const names=fs.readFileSync(new URL('../../../crates/runtime/js/web_apis/css_property_names.js',import.meta.url),'utf8');
const placeholders=vm.runInNewContext(names+'; Array.from(__cssPlaceholderPropertyNames)');
async function evaluate(source){const session=await createSession();try{return await(await session.newPage()).evaluate(source);}finally{await session.close();}}

test('CSS property inventory exposes saved Safari names and defaults as one shared surface',async()=>{
  const expected=inventory.css;
  assert.deepEqual(await evaluate(`(() => {
    const style=document.createElement('div').style,failures=[];
    for(const [name,present]of ${JSON.stringify(expected)}) {
      if((name in style)!==present)failures.push(name+' presence');
      if(present&&style[name]!=='')failures.push(name+' default');
    }
    return failures;
  })()`),[]);
});

test('legacy CSS property placeholders store values without fabricating parsed declarations',async()=>{
  assert.deepEqual(await evaluate(`(() => {
    const e=document.createElement('div'),other=document.createElement('div'),failures=[];
    e.style.width='12px';const original=e.style.cssText;
    for(const name of ${JSON.stringify(placeholders)}) {
      e.style[name]='legacy-value';
      if(e.style[name]!=='legacy-value'||other.style[name]!=='')failures.push(name);
    }
    return [failures,e.style.cssText===original,e.style.length,e.style.width,other.style.length];
  })()`),[[],true,1,'12px',0]);
});

test('native CSS declarations remain canonical through attributes, cloning, stylesheets and custom properties',async()=>{
  assert.deepEqual(await evaluate(`(() => {
    const e=document.createElement('div');e.style.width='120px';e.style.color='red';e.style.fontFamily='serif';
    e.style.setProperty('--theme','blue');e.style.setProperty('height','20px','important');
    const clone=e.cloneNode(),sheet=new CSSStyleSheet();sheet.replaceSync('.card { width: 25px; color: red; }');
    const rule=sheet.cssRules[0];rule.style.width='30px';
    const before=[e.style.width,clone.style.width,e.style.fontFamily==='serif',e.style.getPropertyValue('--theme'),
      e.style.getPropertyPriority('height'),e.getAttribute('style').includes('120px'),rule.style.width];
    e.setAttribute('style','height: 40px;');before.push(e.style.width,e.style.height,clone.style.width);
    e.style.removeProperty('height');before.push(e.style.length);
    return before;
  })()`),['120px','120px',true,'blue','important',true,'30px','','40px','120px',0]);
});
