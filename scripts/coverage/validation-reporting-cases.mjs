export const validationReportingCases=['input','button'].flatMap(tag=>['valid','custom','required','disabled','readonly','barred','datalist'].flatMap(state=>
  ['detached','connected','shadow'].flatMap(root=>['none','cancel','clear','disable','remove','reenter'].map(listener=>({tag,state,root,listener})))));
export function measureValidationReporting(cases){
  return cases.map(({tag,state,root,listener})=>{
    const host=document.createElement('div'),container=root==='shadow'?host.attachShadow({mode:'open'}):host;
    const control=document.createElement(tag),focused=document.createElement('button');focused.id='focused';
    let parent=container;
    if(state==='datalist'){parent=document.createElement('datalist');container.append(parent);}
    parent.append(control);container.append(focused);
    if(root!=='detached'){document.body.append(host);focused.focus();}
    if(state!=='valid'&&state!=='required')control.setCustomValidity('custom');
    if(state==='required')control.setAttribute('required','');if(state==='disabled')control.disabled=true;
    if(state==='readonly')control.setAttribute('readonly','');if(state==='barred')control.type=tag==='input'?'hidden':'reset';
    const events=[];
    container.addEventListener('invalid',e=>events.push(['capture',e.target===control]),true);
    container.addEventListener('invalid',()=>events.push(['bubble']));
    control.addEventListener('invalid',e=>{
      events.push(['target',e.isTrusted,e.bubbles,e.cancelable,e.composed]);
      if(listener==='cancel')e.preventDefault();
      if(listener==='clear'||listener==='reenter'){control.setCustomValidity('');control.removeAttribute('required');}
      if(listener==='disable')control.disabled=true;if(listener==='remove')control.remove();
      if(listener==='reenter')events.push(['nested',control.reportValidity()]);
    },{once:true});
    const result=control.reportValidity();
    const actual=[result,events];host.remove();return actual;
  });
}
