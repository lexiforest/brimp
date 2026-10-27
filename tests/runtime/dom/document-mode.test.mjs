import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createSession} from '../../../scripts/coverage/session.mjs';
import {documentModeCases} from '../../../scripts/coverage/document-mode-cases.mjs';

const fixture=JSON.parse(fs.readFileSync(new URL('../fixtures/document-mode.safari.json',import.meta.url)));

test('DOMParser compatibility modes match installed Safari across syntax and resource boundaries', async () => {
  const cases=documentModeCases();
  assert.deepEqual(fixture.results.map(r=>r.id),cases.map(c=>c.id),'every input requires a Safari observation');
  const session=await createSession();
  try {
    const page=await session.newPage(), failures=[];
    for(let start=0;start<cases.length;start+=40) {
      const expected=fixture.results.slice(start,start+40);
      const actual=await page.evaluate(`(() => {
        const cases=${JSON.stringify(cases.slice(start,start+40))};
        return cases.map(({id,html})=>{
          const doc=new DOMParser().parseFromString(html,'text/html');
          return {id,compatMode:doc.compatMode,cloneMode:doc.cloneNode(false).compatMode};
        });
      })()`);
      for(let i=0;i<actual.length;i++) {
        if(actual[i].compatMode!==expected[i].compatMode || actual[i].cloneMode!==expected[i].compatMode) {
          failures.push({...actual[i],expected:expected[i].compatMode});
        }
      }
    }
    assert.deepEqual(failures,[]);
  } finally {await session.close();}
});

test('DOMParser modes are per document, survive mutation and cloning, and reset on document.write', async () => {
  const session=await createSession();
  try {
    const page=await session.newPage();
    assert.deepEqual(await page.evaluate(`(() => {
      const initial=document.compatMode, failures=[];
      const fast=new DOMParser().parseFromString('<p>x</p>','text/html');
      const slow=new DOMParser().parseFromString('<!--x--><p>x</p>','text/html');
      if(fast.compatMode!=='CSS1Compat' || slow.compatMode!=='BackCompat')failures.push('selection');
      const template=slow.createElement('template');slow.body.appendChild(template);
      if(template.content.ownerDocument.compatMode!=='CSS1Compat' || slow.implementation.createHTMLDocument().compatMode!=='CSS1Compat')failures.push('fresh document inherited quirks');
      fast.__compatMode='BackCompat';slow.__compatMode='CSS1Compat';
      if(fast.compatMode!=='CSS1Compat' || slow.compatMode!=='BackCompat')failures.push('expando changes mode');
      const type=new DOMParser().parseFromString('<!doctype html>','text/html').doctype;
      slow.insertBefore(type,slow.documentElement);
      if(slow.compatMode!=='BackCompat')failures.push('doctype mutation');
      type.remove();
      const saved=fast.cloneNode(true);
      fast.open();fast.write('<p>x</p>');fast.close();
      if(fast.compatMode!=='BackCompat' || saved.compatMode!=='CSS1Compat')failures.push('write/clone');
      fast.open();fast.write('<!doctype html><p>x</p>');fast.close();
      if(fast.compatMode!=='CSS1Compat' || slow.compatMode!=='BackCompat')failures.push('rewrite/isolation');
      if(document.compatMode!==initial)failures.push('main document changed');
      return failures;
    })()`),[]);
  } finally {await session.close();}
});
