import test from 'node:test';
import assert from 'node:assert/strict';
import { createSession } from '../../../scripts/coverage/session.mjs';

// Explicit public IDL/content-attribute pairs, independent of the implementation.
const attributes = `role role
ariaAtomic aria-atomic
ariaAutoComplete aria-autocomplete
ariaBrailleLabel aria-braillelabel
ariaBrailleRoleDescription aria-brailleroledescription
ariaBusy aria-busy
ariaChecked aria-checked
ariaColCount aria-colcount
ariaColIndex aria-colindex
ariaColIndexText aria-colindextext
ariaColSpan aria-colspan
ariaCurrent aria-current
ariaDescription aria-description
ariaDisabled aria-disabled
ariaExpanded aria-expanded
ariaHasPopup aria-haspopup
ariaHidden aria-hidden
ariaInvalid aria-invalid
ariaKeyShortcuts aria-keyshortcuts
ariaLabel aria-label
ariaLevel aria-level
ariaLive aria-live
ariaModal aria-modal
ariaMultiLine aria-multiline
ariaMultiSelectable aria-multiselectable
ariaOrientation aria-orientation
ariaPlaceholder aria-placeholder
ariaPosInSet aria-posinset
ariaPressed aria-pressed
ariaReadOnly aria-readonly
ariaRelevant aria-relevant
ariaRequired aria-required
ariaRoleDescription aria-roledescription
ariaRowCount aria-rowcount
ariaRowIndex aria-rowindex
ariaRowIndexText aria-rowindextext
ariaRowSpan aria-rowspan
ariaSelected aria-selected
ariaSetSize aria-setsize
ariaSort aria-sort
ariaValueMax aria-valuemax
ariaValueMin aria-valuemin
ariaValueNow aria-valuenow
ariaValueText aria-valuetext`
  .split('\n')
  .map((line) => line.split(' '));

async function withPage(run) {
  const session = await createSession();
  try {
    await run(await session.newPage());
  } finally {
    await session.close();
  }
}

test('ARIA nullable strings reflect live content attributes on HTML and SVG elements', async () =>
  withPage(async (page) => {
    const failures = await page.evaluate(`(() => {
    const failures=[];
    for(const svg of [false,true]) for(const [property,attribute] of ${JSON.stringify(attributes)}) {
      const el=svg?document.createElementNS('http://www.w3.org/2000/svg','g'):document.createElement('div');
      const check=(condition,step)=>{if(!condition)failures.push([property,svg,step]);};
      check(el[property]===null,'missing');
      el.setAttribute(attribute,'direct'); check(el[property]==='direct','attribute to IDL');
      el.removeAttribute(attribute); check(el[property]===null,'attribute removal');
      for(const [input,expected] of [['',''],[false,'false'],[42,'42'],['MiXeD invalid','MiXeD invalid']]) {
        el[property]=input;
        check(el[property]===expected && el.getAttributeNS(null,attribute)===expected,'IDL conversion '+expected);
      }
      const attr=el.getAttributeNodeNS(null,attribute);
      attr.value='via Attr'; check(el[property]==='via Attr','live Attr mutation');
      el[property]=null;
      check(el[property]===null && !el.hasAttributeNS(null,attribute),'null removal');
      check(attr.ownerElement===null && attr.value==='via Attr','detached Attr');
      el[property]='again'; el[property]=undefined;
      check(el[property]===null && !el.hasAttributeNS(null,attribute),'undefined removal');
      check(!Object.hasOwn(el,property),'inherited property');
    }
    return failures;
  })()`);
    assert.deepEqual(failures, []);
  }));

test('ARIA descriptors and conversions enforce Web IDL shape and receiver rules', async () =>
  withPage(async (page) => {
    assert.deepEqual(
      await page.evaluate(`(() => {
    const failures=[],el=document.createElement('div');
    for(const [name] of ${JSON.stringify(attributes)}) {
      const d=Object.getOwnPropertyDescriptor(Element.prototype,name);
      if(!d?.enumerable || !d.configurable || d.get.name!=='get '+name || d.set.name!=='set '+name || d.get.length!==0 || d.set.length!==1) failures.push(name+': descriptor');
      for(const receiver of [null,{},document,document.createTextNode('')]) {
        for(const fn of [()=>d.get.call(receiver),()=>d.set.call(receiver,'x')]) {
          try{fn();failures.push(name+': accepted receiver');}catch(e){if(e.name!=='TypeError')failures.push(name+': wrong exception');}
        }
      }
    }
    el.ariaLabel='before';
    try{el.ariaLabel=Symbol('invalid');failures.push('symbol accepted');}catch(e){if(e.name!=='TypeError')failures.push('symbol exception');}
    if(el.ariaLabel!=='before')failures.push('failed conversion mutated attribute');
    const hints=[];
    el.ariaLabel={[Symbol.toPrimitive](hint){hints.push(hint);return 'converted';}};
    if(el.ariaLabel!=='converted' || hints.join()!=='string')failures.push('conversion hint/count');
    const sentinel=new Error('conversion');
    try{el.ariaLabel={toString(){throw sentinel;}};}catch(e){if(e!==sentinel)failures.push('conversion exception identity');}
    if(el.ariaLabel!=='converted')failures.push('throwing conversion mutated attribute');
    let conversions=0;
    el.setAttribute('aria-label',{toString(){conversions++;return 'single conversion';}});
    if(conversions!==1 || el.ariaLabel!=='single conversion')failures.push('setAttribute converted more than once');
    return failures;
  })()`),
      [],
    );
  }));

test('ARIA reflection uses the null namespace and bypasses overridden attribute methods', async () =>
  withPage(async (page) => {
    assert.deepEqual(
      await page.evaluate(`(() => {
    const el=document.createElementNS('http://www.w3.org/2000/svg','g');
    el.setAttributeNS('urn:test','aria-label','namespaced');
    const absent=el.ariaLabel;
    el.ariaLabel='plain';
    const during=[el.ariaLabel,el.getAttributeNS('urn:test','aria-label'),el.getAttributeNS(null,'aria-label')];
    el.ariaLabel=null;
    const after=[el.ariaLabel,el.getAttributeNS('urn:test','aria-label'),el.getAttributeNS(null,'aria-label')];
    for(const name of ['getAttribute','setAttribute','removeAttribute','getAttributeNS','setAttributeNS','removeAttributeNS']) el[name]=()=>{throw Error('override '+name);};
    el.ariaLabel='native'; const overridden=el.ariaLabel; el.ariaLabel=null;
    return {absent,during,after,overridden,removed:el.ariaLabel};
  })()`),
      {
        absent: null,
        during: ['plain', 'namespaced', 'plain'],
        after: [null, 'namespaced', null],
        overridden: 'native',
        removed: null,
      },
    );
  }));

test('ARIA reflection delivers custom-element reactions and attribute mutation records', async () =>
  withPage(async (page) => {
    assert.deepEqual(
      await page.evaluate(`(() => {
    const reactions=[];
    customElements.define('aria-observed',class extends HTMLElement {
      static observedAttributes=['aria-label'];
      attributeChangedCallback(name,oldValue,value,namespace){reactions.push([name,oldValue,value,namespace,this.ariaLabel]);}
    });
    const el=document.createElement('aria-observed');
    const observer=new MutationObserver(()=>{});
    observer.observe(el,{attributes:true,attributeOldValue:true});
    el.ariaLabel='one'; el.ariaLabel='one'; el.ariaLabel=null; el.ariaLabel=undefined;
    el.setAttribute('aria-label','direct');
    const records=observer.takeRecords().map(r=>[r.type,r.attributeName,r.attributeNamespace,r.oldValue,r.target===el]);
    observer.disconnect();
    return {reactions,records};
  })()`),
      {
        reactions: [
          ['aria-label', null, 'one', null, 'one'],
          ['aria-label', 'one', 'one', null, 'one'],
          ['aria-label', 'one', null, null, null],
          ['aria-label', null, 'direct', null, 'direct'],
        ],
        records: [
          ['attributes', 'aria-label', null, null, true],
          ['attributes', 'aria-label', null, 'one', true],
          ['attributes', 'aria-label', null, 'one', true],
          ['attributes', 'aria-label', null, null, true],
        ],
      },
    );
  }));

test('ElementInternals ARIA defaults remain independent of reflected element attributes', async () =>
  withPage(async (page) => {
    assert.deepEqual(
      await page.evaluate(`(() => {
    customElements.define('aria-internals',class extends HTMLElement {});
    const el=document.createElement('aria-internals'),other=document.createElement('aria-internals');
    const internals=el.attachInternals(),second=other.attachInternals();
    const failures=[];
    for(const [name,attribute] of ${JSON.stringify(attributes)}) {
      // Measured in Safari 27.0: these are Element-only accessors.
      if(['ariaColIndexText','ariaDescription','ariaRowIndexText'].includes(name)) {
        if(name in internals || name in ElementInternals.prototype)failures.push(name+': unexpected internals accessor');
        continue;
      }
      if(internals[name]!==null)failures.push(name+': default');
      internals[name]='internal';
      if(el[name]!==null || el.hasAttribute(attribute) || second[name]!==null)failures.push(name+': leaked default');
      el[name]='external';
      if(internals[name]!=='internal')failures.push(name+': changed internal value');
      internals[name]=undefined;
      if(internals[name]!==null || el[name]!=='external')failures.push(name+': removal');
      try{internals[name]=Symbol();failures.push(name+': symbol accepted');}catch(e){if(e.name!=='TypeError')failures.push(name+': symbol exception');}
      const d=Object.getOwnPropertyDescriptor(ElementInternals.prototype,name);
      if(!d.enumerable || !d.configurable)failures.push(name+': descriptor');
    }
    return failures;
  })()`),
      [],
    );
  }));
