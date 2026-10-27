import test from 'node:test';
import assert from 'node:assert/strict';
import {createSession} from '../../../scripts/coverage/session.mjs';

async function evaluate(source) {
  const session = await createSession();
  try { return await (await session.newPage()).evaluate(source); }
  finally { await session.close(); }
}

// Expected values measured in installed Safari 27.0. Property, content attribute,
// missing value, assigned value, reflected value, serialized attribute.
const cases = [
  ['title','title','','Example','Example','Example'],
  ['lang','lang','','ZH-TW','ZH-TW','ZH-TW'],
  ['accessKey','accesskey','','a b','a b','a b'],
  ['dir','dir','','RTL','rtl','RTL'],
  ['enterKeyHint','enterkeyhint','','SEARCH','search','SEARCH'],
  ['inputMode','inputmode','','NUMERIC','numeric','NUMERIC'],
  ['autofocus','autofocus',false,true,true,''],
  ['hidden','hidden',false,'UNTIL-FOUND','until-found','until-found'],
  ['contentEditable','contenteditable','inherit','TRUE','true','true'],
  ['draggable','draggable',false,true,true,'true'],
  ['spellcheck','spellcheck',true,false,false,'false'],
  ['translate','translate',true,false,false,'no'],
];

test('HTML global attributes reflect state with Safari descriptors and receiver checks', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const failures=[];
    for(const [property,attribute,missing,input,expected,raw] of ${JSON.stringify(cases)}) {
      const e=document.createElement('div'),d=Object.getOwnPropertyDescriptor(HTMLElement.prototype,property);
      const check=(ok,step)=>{if(!ok)failures.push(property+': '+step);};
      check(e[property]===missing,'default');e[property]=input;
      check(e[property]===expected && e.getAttribute(attribute)===raw,'setter');
      check(!Object.hasOwn(e,property),'inherited property');
      e.removeAttribute(attribute);check(e[property]===missing,'removal');
      check(d?.enumerable && d.configurable && d.get.name==='get '+property && d.set.name==='set '+property && d.get.length===0 && d.set.length===1,'descriptor');
      for(const receiver of [null,{},document,document.createElementNS('http://www.w3.org/2000/svg','g')]) {
        for(const fn of [()=>d.get.call(receiver),()=>d.set.call(receiver,input)]) {
          try{fn();check(false,'accepted receiver');}catch(error){check(error.name==='TypeError','receiver error');}
        }
      }
    }
    return failures;
  })()`), []);
});

test('HTML string and enum reflections perform single DOMString conversion and preserve raw attributes', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const failures=[];
    for(const [name,attribute] of ${JSON.stringify(cases.slice(0,6))}) {
      const e=document.createElement('div'),hints=[];
      e[name]={[Symbol.toPrimitive](hint){hints.push(hint);return 'RTL';}};
      if(hints.join()!=='string' || e.getAttribute(attribute)!=='RTL')failures.push(name+': conversion');
      try{e[name]=Symbol();failures.push(name+': symbol accepted');}catch(error){if(error.name!=='TypeError')failures.push(name+': symbol error');}
      if(e.getAttribute(attribute)!=='RTL')failures.push(name+': failed write mutated');
      for(const [value,raw] of [[null,'null'],[undefined,'undefined'],[42,'42']]) {
        e[name]=value;if(e.getAttribute(attribute)!==raw)failures.push(name+': DOMString');
      }
    }
    for(const [name,attribute,keywords] of [
      ['dir','dir',['ltr','rtl','auto']],
      ['enterKeyHint','enterkeyhint',['enter','done','go','next','previous','search','send']],
      ['inputMode','inputmode',['none','text','tel','url','email','numeric','decimal','search']],
    ]) {
      const e=document.createElement('div');
      for(const keyword of keywords){e.setAttribute(attribute,keyword.toUpperCase());if(e[name]!==keyword)failures.push(name+': keyword');}
      for(const invalid of ['bogus',' '+keywords[0],keywords[0]+' ','İ']){e.setAttribute(attribute,invalid);if(e[name]!=='')failures.push(name+': invalid keyword');}
    }
    return failures;
  })()`), []);
});

test('contentEditable and hidden implement their distinct keyword and conversion rules', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const failures=[],e=document.createElement('div');
    for(const [raw,expected] of [['','true'],['TRUE','true'],['FALSE','false'],['PLAINTEXT-ONLY','plaintext-only'],['inherit','inherit'],[' true ','inherit']]) {
      e.setAttribute('contenteditable',raw);if(e.contentEditable!==expected)failures.push('contentEditable getter '+raw);
    }
    for(const value of ['TRUE','FALSE','PLAINTEXT-ONLY','INHERIT',true,false]) {
      e.contentEditable=value;
      const expected=String(value).toLowerCase();
      if(e.contentEditable!==expected || e.getAttribute('contenteditable')!==(expected==='inherit'?null:expected))failures.push('contentEditable setter');
    }
    e.contentEditable='true';
    for(const value of ['',null,undefined,' true ','unsupported']) {
      try{e.contentEditable=value;failures.push('invalid contentEditable accepted');}catch(error){if(error.name!=='SyntaxError')failures.push('contentEditable error');}
      if(e.contentEditable!=='true')failures.push('failed conversion changed contentEditable');
    }
    try{e.contentEditable=Symbol();failures.push('contentEditable symbol accepted');}catch(error){if(error.name!=='TypeError')failures.push('contentEditable symbol error');}
    for(const [value,expected,raw] of [[null,false,null],[undefined,false,null],[false,false,null],[0,false,null],[NaN,false,null],['',false,null],[true,true,''],[1,true,''],['false',true,''],['UNTIL-FOUND','until-found','until-found'],[0n,true,''],[{toString(){return ''}},false,null],[{toString(){return 'UNTIL-FOUND'}},'until-found','until-found']]) {
      e.hidden=value;if(e.hidden!==expected || e.getAttribute('hidden')!==raw)failures.push('hidden '+String(value));
    }
    try{e.hidden=Symbol();failures.push('hidden symbol accepted');}catch(error){if(error.name!=='TypeError')failures.push('hidden symbol error');}
    return failures;
  })()`), []);
});

test('spellcheck and translate inherit live state, including Safari namespace and shadow behavior', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const failures=[];
    for(const property of ['spellcheck','translate']) {
      const parent=document.createElement('div'),child=document.createElement('span');parent.appendChild(child);
      parent[property]=false;if(child[property]!==false)failures.push(property+': inherit false');
      child.setAttribute(property,'invalid');if(child[property]!==false)failures.push(property+': invalid inherits');
      child.setAttribute(property,'');if(child[property]!==true)failures.push(property+': empty overrides');
      child.removeAttribute(property);parent[property]=true;if(child[property]!==true)failures.push(property+': live parent');
      parent[property]=false;child.remove();if(child[property]!==true)failures.push(property+': detached default');
      const svg=document.createElementNS('http://www.w3.org/2000/svg','svg');parent.appendChild(svg);svg.appendChild(child);
      svg.setAttribute(property,property==='translate'?'yes':'true');
      if(child[property] !== (property==='spellcheck'))failures.push(property+': non-HTML ancestor');
      const host=document.createElement('div');host[property]=false;host.attachShadow({mode:'open'}).appendChild(child);
      if(child[property] !== (property==='translate'))failures.push(property+': shadow boundary');
    }
    for(const type of ['text','search','url','email','password','number','checkbox']) {
      const e=document.createElement('input'),parent=document.createElement('div');e.type=type;parent.appendChild(e);
      if(!e.spellcheck)failures.push(type+': default');parent.spellcheck=false;if(e.spellcheck)failures.push(type+': inheritance');
    }
    return failures;
  })()`), []);
});

test('draggable auto defaults and boolean setters follow Safari without a rendering engine', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const failures=[];
    for(const [tag,automatic] of [['div',false],['img',true],['a',false],['area',false],['object',false]]) {
      const e=document.createElement(tag);
      if(e.draggable!==automatic)failures.push(tag+': default');
      e.setAttribute('href','');const linked=automatic || tag==='a' || tag==='area';
      for(const raw of ['', 'invalid', ' true ']){e.setAttribute('draggable',raw);if(e.draggable!==linked)failures.push(tag+': auto');}
      e.setAttribute('draggable','FaLsE');if(e.draggable)failures.push(tag+': false');
      e.setAttribute('draggable','TrUe');if(!e.draggable)failures.push(tag+': true');
    }
    for(const [property,on,off] of [['draggable','true','false'],['spellcheck','true','false'],['translate','yes','no'],['autofocus','',null]]) {
      const e=document.createElement('div');
      for(const value of [true,false,'false','',0,1,null,undefined,Symbol(),{toString(){throw Error('must not convert')}}]) {
        e[property]=value;if(e.getAttribute(property)!==(value?on:off))failures.push(property+': boolean conversion');
      }
    }
    return failures;
  })()`), []);
});

test('HTML reflections use null-namespace attributes and deliver custom-element reactions', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const failures=[],reactions=[];
    customElements.define('global-reflections',class extends HTMLElement {
      static observedAttributes=${JSON.stringify(cases.map(c=>c[1]))};
      attributeChangedCallback(...args){reactions.push(args);}
    });
    for(const [property,attribute,missing,input,expected,raw] of ${JSON.stringify(cases)}) {
      const e=document.createElement('global-reflections');
      e.setAttributeNS('urn:test',attribute,'foreign');
      if(e[property]!==missing)failures.push(property+': wrong namespace');
      reactions.length=0;
      const observer=new MutationObserver(()=>{});observer.observe(e,{attributes:true,attributeOldValue:true});
      e[property]=input;e[property]=input;
      const records=observer.takeRecords();observer.disconnect();
      const expectedReactions=property==='autofocus'?[]:[[attribute,null,raw,null],[attribute,raw,raw,null]];
      if(JSON.stringify(reactions)!==JSON.stringify(expectedReactions))failures.push(property+': reactions');
      if(records.length!==2 || records[0].oldValue!==null || records[1].oldValue!==raw || records.some(r=>r.attributeName!==attribute || r.attributeNamespace!==null))failures.push(property+': mutation records');
      for(const method of ['getAttribute','setAttribute','removeAttribute','getAttributeNS','setAttributeNS','removeAttributeNS','hasAttribute'])e[method]=()=>{throw Error('public override')};
      e[property]=input;if(e[property]!==expected)failures.push(property+': public override');
    }
    return failures;
  })()`), []);
});

test('autofocus reactions are delayed, and a synchronous reaction flushes preceding attribute changes', async () => {
  const session=await createSession();
  try {
    const page=await session.newPage();
    assert.deepEqual(await page.evaluate(`(() => {
      window.attributeLog=[];
      customElements.define('autofocus-reactions',class extends HTMLElement {
        static observedAttributes=['autofocus','title'];
        attributeChangedCallback(...args){attributeLog.push(args);}
      });
      const e=document.createElement('autofocus-reactions');
      e.autofocus=true;e.autofocus=false;
      return attributeLog.slice();
    })()`),[]);
    assert.deepEqual(await page.evaluate('attributeLog'),[
      ['autofocus',null,'',null],['autofocus','',null,null],
    ]);
    assert.deepEqual(await page.evaluate(`(() => {
      attributeLog=[];
      const e=document.createElement('autofocus-reactions');
      e.autofocus=true;e.title='x';
      return attributeLog.slice();
    })()`),[['autofocus',null,'',null],['title',null,'x',null]]);
    assert.deepEqual(await page.evaluate(`(() => {
      const log=[];
      customElements.define('nested-attribute-reactions',class extends HTMLElement {
        static observedAttributes=['title','lang'];
        attributeChangedCallback(name){log.push(name);if(name==='title'){this.lang='en';log.push('after lang');}}
      });
      document.createElement('nested-attribute-reactions').title='x';
      return log;
    })()`),['title','lang','after lang']);
  } finally { await session.close(); }
});

test('offset placeholders match Safari detached defaults and read-only API shape', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const failures=[],e=document.createElement('div');
    for(const name of ['offsetHeight','offsetLeft','offsetParent','offsetTop','offsetWidth']) {
      const d=Object.getOwnPropertyDescriptor(HTMLElement.prototype,name),expected=name==='offsetParent'?null:0;
      if(e[name]!==expected || !d?.enumerable || !d.configurable || d.set!==undefined || d.get.name!=='get '+name || d.get.length!==0)failures.push(name+': descriptor/default');
      if(Reflect.set(e,name,123) || e[name]!==expected)failures.push(name+': writable');
      try{d.get.call({});failures.push(name+': receiver');}catch(error){if(error.name!=='TypeError')failures.push(name+': error');}
    }
    return failures;
  })()`), []);
});
