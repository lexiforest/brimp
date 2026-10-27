export const inputRadioGroupCases=['detached','connected','shadow','fragment'].flatMap(context=>
  ['same','empty','case','surrogate','different-forms','explicit-form','unresolved-form','duplicate-id','id-change','name-change','type-change','adopt','insert-fragment','replace','reset-canceled'].map(operation=>({context,operation})));
export function measureInputRadioGroups(cases) {
  return cases.map(({context,operation})=>{
    const host=document.createElement('div');if(context==='connected'||context==='shadow')document.body.append(host);
    const root=context==='shadow'?host.attachShadow({mode:'open'}):context==='fragment'?document.createDocumentFragment():host;
    root.append(document.createElement('form'),document.createElement('form'));
    const[f,g]=root.childNodes;f.id='f';g.id='g';
    let a=document.createElement('input'),b=document.createElement('input');
    for(const i of [a,b]){i.type='radio';i.name='group';}
    f.append(a,b);
    if(operation==='empty')a.name=b.name='';
    if(operation==='case')b.name='GROUP';
    if(operation==='surrogate')a.name=b.name=String.fromCharCode(0xd800);
    if(operation==='different-forms')g.append(b);
    if(operation==='explicit-form'){root.append(b);b.setAttribute('form','f');}
    if(operation==='unresolved-form')b.setAttribute('form','missing');
    if(operation==='duplicate-id'){g.id='f';const div=document.createElement('div');div.id='f';root.prepend(div);a.setAttribute('form','f');}
    if(operation==='id-change')b.setAttribute('form','g');
    if(operation==='name-change')b.name='other';
    if(operation==='type-change')b.type='checkbox';
    a.checked=true;b.checked=true;
    const snapshot=()=>[a.checked,b.checked,a.form===b.form,a.form===f,b.form===g];
    const result=[snapshot()];
    if(operation==='id-change')g.id='other';
    else if(operation==='name-change')b.name='group';
    else if(operation==='type-change')b.type='radio';
    else if(operation==='adopt'){document.implementation.createHTMLDocument().adoptNode(b);f.append(b);}
    else if(operation==='insert-fragment'){
      const fragment=document.createDocumentFragment();fragment.append(a,b);a.checked=true;b.checked=true;f.append(fragment);
    }else if(operation==='replace'){const c=b.cloneNode();c.checked=true;f.replaceChild(c,a);a=c;}
    else if(operation==='reset-canceled'){f.addEventListener('reset',e=>e.preventDefault());f.reset();}
    else {a.checked=false;a.checked=true;}
    result.push(snapshot());host.remove();return result;
  });
}
