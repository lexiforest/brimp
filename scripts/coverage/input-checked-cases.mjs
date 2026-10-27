const contexts=['detached','connected','form','fragment','template','shadow','shadow-detached','parsed'];
const setups=['markup','script-defaults','script-checked'];
const operations=['none','a-true','a-toggle','b-false-default','a-default-toggle','b-default-toggle',
  'a-attribute-value','b-name-away-back','b-type-away-back','move-a-last','remove-append-a','reset'];
export const inputCheckedCases=contexts.flatMap(context=>setups.flatMap(setup=>operations.map(operation=>({context,setup,operation}))));
export function measureInputChecked(cases) {
  const html='<input type="radio" name="g" checked><input type="radio" name="g" checked>';
  return cases.map(({context,setup,operation})=>{
    let root,cleanup=null;
    if(context==='parsed')root=new DOMParser().parseFromString(setup==='markup'?html:'','text/html').body;
    else if(context==='fragment')root=document.createDocumentFragment();
    else if(context==='template')root=document.createElement('template').content;
    else if(context.startsWith('shadow')){
      cleanup=document.createElement('div');if(context==='shadow')document.body.append(cleanup);root=cleanup.attachShadow({mode:'open'});
    }else{
      root=document.createElement(context==='form'?'form':'div');
      if(context==='connected'){document.body.append(root);cleanup=root;}
    }
    if(setup==='markup'&&context!=='parsed') {
      if(context==='fragment'||context==='template'){
        const template=document.createElement('template');template.innerHTML=html;root.append(...template.content.childNodes);
      }else root.innerHTML=html;
    }else if(setup!=='markup'){
      for(let index=0;index<2;index++){
        const input=document.createElement('input');input.type='radio';input.name='g';
        if(setup==='script-defaults')input.defaultChecked=true;else input.checked=true;
        root.append(input);
      }
    }
    const [a,b]=root.querySelectorAll('input');
    const snapshot=()=>[a.checked,b.checked,a.defaultChecked,b.defaultChecked];
    const results=[snapshot()];
    if(operation==='a-true')a.checked=true;
    if(operation==='a-toggle'){a.checked=false;a.checked=true;}
    if(operation==='b-false-default'){b.checked=false;b.defaultChecked=false;b.defaultChecked=true;}
    if(operation==='a-default-toggle'){a.defaultChecked=false;a.defaultChecked=true;}
    if(operation==='b-default-toggle'){b.defaultChecked=false;b.defaultChecked=true;}
    if(operation==='a-attribute-value')a.setAttribute('checked','new');
    if(operation==='b-name-away-back'){b.name='other';b.name='g';}
    if(operation==='b-type-away-back'){b.type='checkbox';b.type='radio';}
    if(operation==='move-a-last')root.append(a);
    if(operation==='remove-append-a'){a.remove();root.append(a);}
    if(operation==='reset'){
      if(root.localName==='form')root.reset();
      else{const form=document.createElement('form');root.append(form);form.append(a,b);form.reset();}
    }
    results.push(snapshot());
    a.defaultChecked=false;a.defaultChecked=true;b.defaultChecked=false;b.defaultChecked=true;
    results.push(snapshot());
    cleanup?.remove();
    return results;
  });
}
