import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createSession} from '../../../scripts/coverage/session.mjs';
import {submitterActionCases as cases, measureSubmitterActions as measure} from '../../../scripts/coverage/submitter-action-cases.mjs';

async function evaluate(source) {
  const session=await createSession();
  try { return await (await session.newPage()).evaluate(source); }
  finally { await session.close(); }
}

for (const tag of ['input','button']) {
const interfaceName=tag==='input'?'HTMLInputElement':'HTMLButtonElement';
test(`${tag} formAction parsing and USVString storage match 308 measured Safari cases`, async () => {
  const reference=JSON.parse(fs.readFileSync(new URL(`./fixtures/${tag}-action.safari.json`,import.meta.url)));
  assert.deepEqual(reference.cases.map(({expected,...c})=>c),cases);
  const actual=JSON.parse(await evaluate(`JSON.stringify((${measure})(${JSON.stringify(cases)},${JSON.stringify(tag)}))`));
  assert.deepEqual(cases.flatMap((c,i)=>JSON.stringify(actual[i])===JSON.stringify(reference.cases[i].expected)?[]:
    [{...c,actual:actual[i],expected:reference.cases[i].expected}]),[]);
});

test(`${tag} formMethod and formEnctype distinguish missing, invalid and canonical keywords`, async () => {
  assert.deepEqual(await evaluate(`(() => {
    const failures=[];
    for(const [name,keywords,invalid] of [['formMethod',['get','post','dialog'],'get'],
      ['formEnctype',['application/x-www-form-urlencoded','multipart/form-data','text/plain'],'application/x-www-form-urlencoded']]) {
      const input=document.createElement('${tag}'),attribute=name.toLowerCase(),form=document.createElement('form');
      form.method='post';form.enctype='multipart/form-data';form.append(input);
      if(input[name]!=='')failures.push([name,'missing']);
      for(const value of [...keywords,...keywords.map(s=>s.toUpperCase()),'',null,undefined,1n,0,{},' get','post ',
        'poſt','DİALOG','text/plain; charset=UTF-8','multipart/form-data\t',String.fromCharCode(0xd800)]) {
        input[name]=value;const raw=String(value),lower=raw.replace(/[A-Z]/g,c=>c.toLowerCase());
        const expected=keywords.includes(lower)?lower:invalid;
        if(input[name]!==expected||input.getAttribute(attribute)!==raw)failures.push([name,'conversion',raw]);
        for(const type of ${JSON.stringify(tag==='input'?['text','submit','image','reset','checkbox','file']:['submit','reset','button'])}){input.type=type;if(input[name]!==expected)failures.push([name,type]);}
      }
      input.removeAttribute(attribute);input.setAttributeNS('urn:foreign',attribute,keywords[1]);
      if(input[name]!=='')failures.push([name,'namespace']);
      input[name]=keywords[2];if(input.getAttributeNS('urn:foreign',attribute)!==keywords[1]||input.getAttributeNS(null,attribute)!==keywords[2])failures.push([name,'identity']);
      const clone=input.cloneNode();input[name]=keywords[0];if(clone[name]!==keywords[2])failures.push([name,'clone']);
      input.getAttribute=input.setAttribute=input.hasAttribute=input.removeAttribute=()=>{throw Error('override');};
      input[name]=keywords[1];if(input[name]!==keywords[1])failures.push([name,'canonical']);
    }
    return failures;
  })()`),[]);
});

test(`${tag} form override descriptors, conversion failures and mutation records follow Safari`, async () => {
  assert.deepEqual(await evaluate(`(() => {
    const failures=[];
    for(const name of ['formMethod','formEnctype','formAction']) {
      const i=document.createElement('${tag}'),attribute=name.toLowerCase(),d=Object.getOwnPropertyDescriptor(${interfaceName}.prototype,name);
      if(!d.enumerable||!d.configurable||d.get.name!=='get '+name||d.set.name!=='set '+name||d.get.length!==0||d.set.length!==1)failures.push([name,'descriptor']);
      let calls=[];const value={[Symbol.toPrimitive](hint){calls.push(hint);return 'https://example.test/one';}};
      d.set.call(i,value);if(calls.join()!=='string')failures.push([name,'hint']);
      for(const receiver of [{},document.createElement('div'),Object.create(${interfaceName}.prototype)]) {
        calls=[];try{d.set.call(receiver,value);failures.push([name,'brand']);}catch(e){if(e.name!=='TypeError')failures.push([name,e.name]);}
        if(calls.length)failures.push([name,'brand before conversion']);
        try{d.get.call(receiver);failures.push([name,'getter brand']);}catch(e){if(e.name!=='TypeError')failures.push([name,e.name]);}
      }
      const before=i.getAttribute(attribute),observer=new MutationObserver(()=>{});observer.observe(i,{attributes:true,attributeOldValue:true});
      for(const bad of [Symbol(),{[Symbol.toPrimitive](){throw new RangeError('conversion');}}]) {
        try{i[name]=bad;failures.push([name,'expected throw']);}catch(e){if(!['TypeError','RangeError'].includes(e.name))failures.push([name,e.name]);}
      }
      if(i.getAttribute(attribute)!==before||observer.takeRecords().length)failures.push([name,'failed conversion mutated']);
      i[name]='post';i[name]='post';i[name]='dialog';
      const records=observer.takeRecords().map(r=>[r.attributeName,r.attributeNamespace,r.oldValue]);observer.disconnect();
      if(JSON.stringify(records)!==JSON.stringify([[attribute,null,before],[attribute,null,'post'],[attribute,null,'post']]))failures.push([name,'records']);
    }
    return failures;
  })()`),[]);
});

test(`${tag} formAction follows document ownership, live bases, namespaces and adoption`, async () => {
  assert.deepEqual(await evaluate(`(() => {
    const failures=[],current=document.createElement('${tag}');
    if(current.formAction!==document.URL)failures.push('current document default');
    const owner=document.implementation.createHTMLDocument(),other=document.implementation.createHTMLDocument();
    const input=owner.createElement('${tag}');owner.body.append(input);
    if(owner.URL!=='about:blank'||input.formAction!=='')failures.push('created document defaults');
    const base=owner.createElement('base');base.setAttribute('href','https://one.test/a/');owner.head.append(base);
    input.formAction='';if(input.formAction!=='')failures.push('empty ignores base');
    input.formAction='go';if(input.formAction!=='https://one.test/a/go')failures.push('relative');
    base.setAttribute('href','https://two.test/b/');if(input.formAction!=='https://two.test/b/go')failures.push('live base');
    const clone=owner.cloneNode(true);if(clone.querySelector('${tag}').formAction!=='https://two.test/b/go')failures.push('document clone');
    const otherBase=other.createElement('base');otherBase.href='https://other.test/c/';other.head.append(otherBase);
    const imported=other.importNode(input,true);if(imported.formAction!=='https://other.test/c/go')failures.push('import');
    other.adoptNode(input);if(input.formAction!=='https://other.test/c/go')failures.push('adopt');
    input.removeAttribute('formaction');input.setAttributeNS('urn:foreign','formaction','https://foreign.test/');
    if(input.formAction!=='')failures.push('foreign namespace default');input.formAction='ok';
    if(input.formAction!=='https://other.test/c/ok'||input.getAttributeNS('urn:foreign','formaction')!=='https://foreign.test/')failures.push('namespace identity');
    Object.defineProperty(input,'ownerDocument',{value:owner});Object.defineProperty(other,'URL',{value:'https://wrong.test/'});
    Object.defineProperty(otherBase,'href',{value:'https://wrong.test/'});input.getAttribute=input.setAttribute=input.hasAttribute=()=>{throw Error('override');};
    input.formAction='canonical';if(input.formAction!=='https://other.test/c/canonical')failures.push('canonical URLs');
    const template=owner.createElement('template');template.innerHTML='<${tag}>';const inside=template.content.firstChild;
    if(inside.ownerDocument.URL!=='about:blank'||inside.formAction!=='about:blank')failures.push('template default');
    inside.formAction='go';if(inside.formAction!=='go')failures.push('template is separate document');
    const parsed=new DOMParser().parseFromString('<${tag}>','text/html');
    if(parsed.querySelector('${tag}').formAction!==document.URL)failures.push('parsed document URL');
    return failures;
  })()`),[]);
});

test(`${tag} formAction uses only the first eligible HTML base`, async () => {
  assert.deepEqual(await evaluate(`(() => {
    const failures=[],owner=document.implementation.createHTMLDocument(),i=owner.createElement('${tag}');i.formAction='go';
    const svg=owner.createElementNS('http://www.w3.org/2000/svg','base');svg.setAttribute('href','https://svg.test/');owner.head.append(svg);
    const namespaced=owner.createElement('base');namespaced.setAttributeNS('urn:test','href','https://foreign.test/');owner.head.append(namespaced);
    const first=owner.createElement('base');first.setAttribute('href','http://[bad');owner.head.append(first);
    const second=owner.createElement('base');second.href='https://second.test/';owner.head.append(second);
    if(i.formAction!=='go')failures.push('invalid first base blocks second');
    first.remove();if(i.formAction!=='https://second.test/go')failures.push('eligible base');
    return failures;
  })()`),[]);
});
}
