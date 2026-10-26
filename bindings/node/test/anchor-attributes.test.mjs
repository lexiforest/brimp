import test from'node:test';import assert from'node:assert/strict';import fs from'node:fs';
import{createSession}from'../../../scripts/coverage/session.mjs';import{anchorStringFields,anchorAttributeCases as cases,measureAnchorAttributes as measure}from'../../../scripts/coverage/anchor-attribute-cases.mjs';
async function evaluate(source){const s=await createSession();try{return await(await s.newPage()).evaluate(source);}finally{await s.close()}}

test('anchor attributes and text match 208 Safari conversion and mutation cases',async()=>{
 const ref=JSON.parse(fs.readFileSync(new URL('./fixtures/anchor-attributes.safari.json',import.meta.url))).cases;assert.deepEqual(ref.map(({expected,...c})=>c),cases);
 const actual=JSON.parse(await evaluate(`JSON.stringify((${measure})(${JSON.stringify(cases)}))`));
 assert.deepEqual(ref.flatMap((c,i)=>JSON.stringify(c.expected)===JSON.stringify(actual[i])?[]:[{...c,actual:actual[i]}]),[]);
});

test('anchor reflections use canonical null-namespace attributes, preserve UTF-16, and have Safari descriptors',async()=>{
 assert.deepEqual(await evaluate(`(()=>{
 const failures=[];
 for(const name of ${JSON.stringify(anchorStringFields)}){
  const a=document.createElement('a'),d=Object.getOwnPropertyDescriptor(HTMLAnchorElement.prototype,name);
  if(a[name]!=='')failures.push([name,'missing']);
  if(!d.enumerable||!d.configurable||d.get.name!=='get '+name||d.set.name!=='set '+name||d.get.length!==0||d.set.length!==1)failures.push([name,'descriptor']);
  for(const receiver of [{},document.createElement('div'),Object.create(HTMLAnchorElement.prototype)]){
   let converted=false;try{d.set.call(receiver,{toString(){converted=true;return 'x'}});failures.push('receiver')}catch(e){if(e.name!=='TypeError'||converted)failures.push('brand before conversion')}
   try{d.get.call(receiver);failures.push('getter receiver')}catch(e){if(e.name!=='TypeError')failures.push(e.name)}
  }
  const attribute=name.toLowerCase();a.setAttributeNS('urn:foreign',attribute,'foreign');if(a[name]!=='')failures.push('namespace');
  const source='a'+String.fromCharCode(0xd800)+'b'+String.fromCharCode(0xdc00),raw=name==='ping'?'a�b�':source;a[name]=source;
  if(a.getAttributeNS(null,attribute)!==raw||a.getAttributeNS('urn:foreign',attribute)!=='foreign')failures.push('attribute identity');
  const clone=a.cloneNode();a[name]='changed';if(clone[name]!==raw)failures.push('clone');
  document.implementation.createHTMLDocument().adoptNode(clone);if(clone[name]!==raw)failures.push('adoption');
  // Safari preserves raw attribute code units in ping's getter, despite its scalar-converting setter.
  a.setAttributeNS(null,attribute,source);if(a[name]!==source)failures.push('raw attribute getter');
  a.getAttribute=a.setAttribute=a.hasAttribute=()=>{throw Error('override')};a[name]='canonical';if(a[name]!=='canonical')failures.push('canonical');
 }
 return failures;
 })()`),[]);
});

test('anchor referrerPolicy reflects Safari keywords without trimming or accepting legacy aliases',async()=>{
 assert.deepEqual(await evaluate(`(()=>{
 const failures=[],a=document.createElement('a');const words=['','no-referrer','no-referrer-when-downgrade','same-origin','origin','strict-origin','origin-when-cross-origin','strict-origin-when-cross-origin','unsafe-url'];
 for(const value of [...words,...words.map(s=>s.toUpperCase()),' same-origin','origin ','always','never','default','ſame-origin','ORİGIN',null,undefined]){
  a.referrerPolicy=value;const raw=String(value),lower=raw.replace(/[A-Z]/g,c=>c.toLowerCase());
  if(a.referrerPolicy!==(words.includes(lower)?lower:'')||a.getAttribute('referrerpolicy')!==raw)failures.push(raw);
 }
 a.removeAttribute('referrerpolicy');a.setAttributeNS('urn:other','referrerpolicy','origin');if(a.referrerPolicy!=='')failures.push('namespace');
 a.getAttribute=a.setAttribute=()=>{throw Error('override')};a.referrerPolicy='ORIGIN';if(a.referrerPolicy!=='origin')failures.push('canonical');
 if('hrefTranslate' in a||Object.getOwnPropertyDescriptor(HTMLAnchorElement.prototype,'hrefTranslate')!==undefined)failures.push('unsupported hrefTranslate');
 return failures;
 })()`),[]);
});

test('anchor text is canonical descendant text and replaces children independently of the text content attribute',async()=>{
 assert.deepEqual(await evaluate(`(()=>{
 const failures=[],a=document.createElement('a');
 a.innerHTML='<span>one</span><!--ignored--><script>two</script><style>three</style><template>ignored</template>four';
 if(a.text!=='onetwothreefour')failures.push('descendants');
 a.setAttribute('text','attribute');Object.defineProperty(a,'textContent',{get(){throw Error('getter')},set(){throw Error('setter')}});
 a.text='plain';if(a.text!=='plain'||a.childNodes.length!==1||a.firstChild.nodeType!==3||a.getAttribute('text')!=='attribute')failures.push('canonical text');
 a.text='';if(a.text!==''||a.childNodes.length!==0)failures.push('empty');
 const d=Object.getOwnPropertyDescriptor(HTMLAnchorElement.prototype,'text');
 if(!d.enumerable||!d.configurable||d.get.name!=='get text'||d.set.name!=='set text'||d.get.length!==0||d.set.length!==1)failures.push('descriptor');
 for(const receiver of [{},document.createElement('div'),Object.create(HTMLAnchorElement.prototype)]){
  let converted=false;try{d.set.call(receiver,{toString(){converted=true;return 'x'}});failures.push('receiver')}catch(e){if(e.name!=='TypeError'||converted)failures.push('brand before conversion')}
 }
 return failures;
 })()`),[]);
});
