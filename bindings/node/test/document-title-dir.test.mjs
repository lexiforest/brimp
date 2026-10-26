import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createSession} from '../../../scripts/coverage/session.mjs';
import {documentTitleCases, documentDirCases, measureDocumentTitleDir} from '../../../scripts/coverage/document-title-dir-cases.mjs';

const reference = JSON.parse(fs.readFileSync(new URL('./fixtures/document-title-dir.safari.json', import.meta.url)));
for (const [property, cases] of [['title', documentTitleCases], ['dir', documentDirCases]]) {
  test(`Document.${property} matches Safari selection, conversion, namespaces and mutations`, async () => {
    assert.deepEqual(reference[property].map(({expected, ...c}) => c), cases);
    const session = await createSession();
    try {
      const page = await session.newPage();
      const actual = JSON.parse(await page.evaluate(`JSON.stringify((${measureDocumentTitleDir})(${JSON.stringify(cases)},${JSON.stringify(property)}))`));
      const differences = reference[property].flatMap((c, i) => JSON.stringify(c.expected) === JSON.stringify(actual[i]) ? [] : [{...c, actual: actual[i]}]);
      assert.deepEqual(differences.slice(0, 8), [], `${differences.length} of ${cases.length} cases differ`);
    } finally { await session.close(); }
  });
}

test('Document title creation and direction use case-sensitive local names and namespaces', async () => {
  const session = await createSession();
  try {
    assert.deepEqual(await (await session.newPage()).evaluate(`(() => {
      const H='http://www.w3.org/1999/xhtml', S='http://www.w3.org/2000/svg', failures=[];
      for(const [ns,name] of [[H,'html'],[H,'div'],[H,'HTML'],[H,'x:html'],[S,'svg'],[S,'x:svg'],[S,'SVG']]) {
        for(const mode of ['direct','nested','foreign','upper','prefixed']) {
          const d=new Document(), root=d.createElementNS(ns,name);d.appendChild(root);
          const head=d.createElementNS(mode==='foreign'?null:H,mode==='upper'?'HEAD':mode==='prefixed'?'x:head':'head');
          if(mode==='nested'){const box=d.createElementNS(H,'div');root.appendChild(box);box.appendChild(head)}else root.appendChild(head);
          d.title='new';d.dir='RTL';
          const svg=ns===S&&root.localName==='svg', created=svg||ns===H&&['direct','prefixed'].includes(mode);
          if(d.title!==(created?'new':''))failures.push([name,mode,'title',d.title]);
          const parent=svg?root:head, title=parent.firstChild;
          if(created&&(!title||title.localName!=='title'||title.namespaceURI!==ns||title.textContent!=='new'))failures.push([name,mode,'creation']);
          const directional=ns===H&&root.localName==='html';
          if(d.dir!==(directional?'rtl':'')||root.getAttribute('dir')!==(directional?'RTL':null))failures.push([name,mode,'dir']);
        }
      }
      return failures;
    })()`), []);
  } finally { await session.close(); }
});

test('Document title and dir have Safari descriptors, native receivers and conversion before selection', async () => {
  const session = await createSession();
  try {
    assert.deepEqual(await (await session.newPage()).evaluate(`(() => {
      const failures=[];
      for(const name of ['title','dir']) {
        const {get,set,enumerable,configurable}=Object.getOwnPropertyDescriptor(Document.prototype,name);
        if(!enumerable||!configurable||get.name!=='get '+name||set.name!=='set '+name||get.length!==0||set.length!==1)failures.push([name,'descriptor']);
        const forged=document.createElement('div');Object.setPrototypeOf(forged,Document.prototype);
        let conversions=0;
        for(const receiver of [null,{},Object.create(Document.prototype),forged]) {
          for(const fn of [get,set])try{fn.call(receiver,{toString(){conversions++;return 'rtl'}});failures.push([name,'receiver accepted'])}catch(e){if(e.name!=='TypeError')failures.push([name,e.name])}
        }
        if(conversions!==0)failures.push([name,'receiver conversion']);
        const d=document.implementation.createHTMLDocument('old'),oldRoot=d.documentElement;
        const replacement=document.implementation.createHTMLDocument('replacement').documentElement;
        set.call(d,{toString(){d.replaceChild(replacement,oldRoot);return name==='dir'?'RTL':'new'}});
        if(get.call(d)!==(name==='dir'?'rtl':'new'))failures.push([name,'conversion selection']);
        Object.setPrototypeOf(d,null);
        set.call(d,name==='dir'?'auto':'native');
        if(get.call(d)!==(name==='dir'?'auto':'native'))failures.push([name,'native identity']);
      }
      return failures;
    })()`), []);
  } finally { await session.close(); }
});

test('Document title and dir ignore overridden public DOM properties', async () => {
  const session = await createSession();
  try {
    assert.deepEqual(await (await session.newPage()).evaluate(`(() => {
      const d=document.implementation.createHTMLDocument('old'),root=d.documentElement,head=d.head,title=head.querySelector('title');
      const observer=new MutationObserver(()=>{});observer.observe(d,{subtree:true,childList:true,attributes:true,attributeOldValue:true});
      for(const [object,names] of [[d,['documentElement','head','querySelector','querySelectorAll','createElement','createElementNS']], [root,['namespaceURI','localName','getAttribute','getAttributeNS','setAttribute','setAttributeNS']], [title,['text','textContent']]]) {
        for(const name of names)Object.defineProperty(object,name,{configurable:true,get(){throw Error('public '+name)}});
      }
      const failures=[];
      try{d.title=' new title ';d.dir='RTL';if(d.title!=='new title'||d.dir!=='rtl')failures.push('value')}catch(e){failures.push(e.message)}
      const records=observer.takeRecords();observer.disconnect();
      if(records.length!==2||records[0].target!==title||records[1].target!==root)failures.push('records');
      return failures;
    })()`), []);
  } finally { await session.close(); }
});

test('Document title follows moved titles and replaces text even for identical nonempty writes', async () => {
  const session = await createSession();
  try {
    assert.deepEqual(await (await session.newPage()).evaluate(`(() => {
      const d=document.implementation.createHTMLDocument('one'),a=d.head.firstChild,b=d.createElement('title'),out=[];
      b.textContent='two';d.body.appendChild(b);out.push(d.title);
      d.head.insertBefore(b,a);out.push(d.title);d.title='changed';out.push(a.text,b.text);
      b.remove();out.push(d.title);a.remove();out.push(d.title);d.body.appendChild(a);out.push(d.title);
      for(const value of ['one','one','','']) {
        const previous=a.firstChild,range=d.createRange();
        if(previous)range.setStart(previous,1);else range.setStart(a,0);range.collapse(true);
        const observer=new MutationObserver(()=>{});observer.observe(a,{childList:true,characterData:true});
        d.title=value;
        out.push([value,a.firstChild===previous,range.startContainer===a,range.startOffset,observer.takeRecords().map(r=>[r.type,r.addedNodes.length,r.removedNodes.length])]);
        observer.disconnect();
      }
      return out;
    })()`), ['one','two','one','changed','one','','one',
      ['one',false,true,0,[['childList',1,1]]],['one',false,true,0,[['childList',1,1]]],
      ['',false,true,0,[['childList',0,1]]],['',true,true,0,[]]]);
  } finally { await session.close(); }
});
