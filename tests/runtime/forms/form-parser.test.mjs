import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createSession} from '../../../scripts/coverage/session.mjs';
import {formParserCases,measureFormParser,parserRadioCases,measureParserRadios,parserRadioMutationCases,measureParserRadioMutations} from '../../../scripts/coverage/form-parser-cases.mjs';
async function evaluate(source){const session=await createSession();try{return await(await session.newPage()).evaluate(source);}finally{await session.close();}}
test('parser-created form owners and mutation resets match 245 Safari sequences',async()=>{
  const ref=JSON.parse(fs.readFileSync(new URL('../fixtures/form-parser.safari.json',import.meta.url)));
  assert.deepEqual(ref.cases.map(({expected,...c})=>c),formParserCases);
  const actual=JSON.parse(await evaluate(`JSON.stringify((${measureFormParser})(${JSON.stringify(formParserCases)}))`));
  assert.deepEqual(formParserCases.flatMap((c,i)=>JSON.stringify(actual[i])===JSON.stringify(ref.cases[i].expected)?[]:[{...c,actual:actual[i],expected:ref.cases[i].expected}]),[]);
});
test('parser-owned radio groups stay separate after their non-ancestor form closes',async()=>{
  const ref=JSON.parse(fs.readFileSync(new URL('../fixtures/form-parser.safari.json',import.meta.url)));
  assert.deepEqual(ref.radio_cases.map(c=>c.kind),parserRadioCases);
  assert.deepEqual(await evaluate(`(${measureParserRadios})(${JSON.stringify(parserRadioCases)})`),ref.radio_cases.map(c=>c.expected));
});

test('parser owners survive moving a common ancestor and adoption but reset when separated or cloned',async()=>{
  assert.deepEqual(await evaluate(`(() => {
    const failures=[];
    for(const action of ['move-control','move-cell','move-table','move-form','remove-reinsert-form','adopt','form-attribute','remove-missing-form','foreign-form','remove-id','same-id']){
      const doc=new DOMParser().parseFromString('<div id=root><table><form id=f><tr><td><input id=i></table></form></div>','text/html');
      const input=doc.getElementById('i'),form=doc.getElementById('f'),root=doc.getElementById('root');
      if(action==='move-control')input.parentNode.append(input);
      if(action==='move-cell')input.parentNode.parentNode.append(input.parentNode);
      if(action==='move-table')root.append(root.firstChild);
      if(action==='move-form')form.parentNode.append(form);
      if(action==='remove-reinsert-form'){const parent=form.parentNode;form.remove();parent.prepend(form);}
      if(action==='adopt'){document.adoptNode(root);document.body.append(root);}
      if(action==='form-attribute')input.setAttribute('form','');
      if(action==='remove-missing-form')input.removeAttribute('form');
      if(action==='foreign-form')input.setAttributeNS('urn:x','form','x');
      if(action==='remove-id')form.removeAttribute('id');
      if(action==='same-id')form.id='f';
      const keep=['move-table','adopt','remove-missing-form','foreign-form','remove-id','same-id'].includes(action);
      if((input.form===form)!==keep)failures.push(action);
      if(root.cloneNode(true).querySelector('input')?.form)failures.push([action,'clone']);root.remove();
    }
    for(const kind of ['div','template','range']){
      let root=document.createElement(kind==='range'?'div':kind);
      const source='<table><form id=f><tr><td><input id=i></table></form>';
      if(kind==='range'){const range=document.createRange();range.selectNodeContents(root);root=range.createContextualFragment(source);}
      else{root.innerHTML=source;if(kind==='template')root=root.content;}
      if(root.querySelector('input').form!==null)failures.push([kind,'fragment owner']);
    }
    return failures;
  })()`),[]);
});

test('form properties are canonical readonly descriptors and honor exact connected form IDs',async()=>{
  assert.deepEqual(await evaluate(`(() => {
    const failures=[];
    for(const [tag,ctor] of [['input',HTMLInputElement],['button',HTMLButtonElement],['select',HTMLSelectElement],['textarea',HTMLTextAreaElement],['output',HTMLOutputElement],['object',HTMLObjectElement],['fieldset',HTMLFieldSetElement]]){
      const host=document.createElement('div');document.body.append(host);
      const form=document.createElement('form'),other=document.createElement('form'),control=document.createElement(tag);host.append(form,other);form.append(control);form.id='f';other.id='other';
      const d=Object.getOwnPropertyDescriptor(ctor.prototype,'form');
      if(!d.enumerable||!d.configurable||d.set||d.get.name!=='get form')failures.push([tag,'descriptor']);
      try{d.get.call({});failures.push([tag,'brand']);}catch(e){if(e.name!=='TypeError')failures.push(e.name);}
      control.setAttribute('form','other');if(control.form!==other)failures.push([tag,'explicit']);
      const blocker=document.createElement('div');blocker.id='other';host.prepend(blocker);if(control.form!==null)failures.push([tag,'first ID']);blocker.remove();
      for(const id of ['', ' spaced ',String.fromCharCode(0xd800)]){other.id=id;control.setAttribute('form',id);if(control.form!==(id===''?null:other))failures.push([tag,'exact ID']);}
      control.removeAttribute('form');control.setAttributeNS('urn:x','form','missing');if(control.form!==form)failures.push([tag,'namespace']);
      control.getAttribute=control.closest=control.getRootNode=()=>{throw Error('override');};Object.defineProperty(form,'id',{value:'wrong'});
      if(control.form!==form)failures.push([tag,'canonical']);
      host.remove();if(control.form!==form)failures.push([tag,'detached ancestor']);
      control.setAttribute('form','missing');if(control.form!==form)failures.push([tag,'detached form attribute']);
    }
    return failures;
  })()`),[]);
});

test('document.write retains parser form owners across chunks and resets them on replacement',async()=>{
  assert.deepEqual(await evaluate(`(() => {
    const failures=[],doc=document.implementation.createHTMLDocument();doc.open();
    doc.write('<table><form id=f><tr><td><input type=radio name=g id=a checked>');
    const a=doc.getElementById('a'),form=doc.getElementById('f');
    if(a.form!==form||!a.checked)failures.push('first chunk');
    doc.write('</table></form><input type=radio name=g id=b checked>');doc.close();
    const b=doc.getElementById('b');if(a.form!==form||b.form!==null||!a.checked||!b.checked)failures.push('second chunk');
    const retained=doc.documentElement;doc.open();doc.write('<input id=fresh>');doc.close();
    if(doc.getElementById('fresh').form!==null||a.form!==form||retained.parentNode!==null)failures.push('replacement');
    return failures;
  })()`),[]);
});

test('radio mutations preserve Safari grouping when parser form ownership changes',async()=>{
  const ref=JSON.parse(fs.readFileSync(new URL('../fixtures/form-parser.safari.json',import.meta.url)));
  assert.deepEqual(ref.radio_mutations.map(c=>c.operation),parserRadioMutationCases);
  assert.deepEqual(await evaluate(`(${measureParserRadioMutations})(${JSON.stringify(parserRadioMutationCases)})`),ref.radio_mutations.map(c=>c.expected));
});
