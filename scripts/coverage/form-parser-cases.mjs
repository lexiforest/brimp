// Parser-owned associations can survive after table parsing separates the form
// and control in the resulting tree. These references track the remaining work.
export const formParserCases=['input','button','select','textarea','output','object','fieldset'].flatMap(tag=>
  ['ancestor','table-form','form-before-table','explicit','missing-explicit'].flatMap(markup=>
  ['none','control-id','class','form-id','form-attribute','reinsert','remove-form'].map(operation=>({tag,markup,operation}))));
export function measureFormParser(cases) {
  return cases.map(({tag,markup,operation})=>{
    const attribute=markup==='explicit'?' form=f':markup==='missing-explicit'?' form=missing':'';
    const control='<'+tag+' id=i'+attribute+'></'+tag+'>';
    const source=markup==='ancestor'?'<form id=f>'+control+'</form>':markup==='form-before-table'?'<form id=f><table><tr><td>'+control+'</table></form>':'<table><form id=f><tr><td>'+control+'</table>';
    const doc=new DOMParser().parseFromString(source+'<label for=i></label>','text/html');
    const input=doc.getElementById('i'),form=doc.getElementById('f'),label=doc.querySelector('label');
    const snapshot=()=>[input.form===form,label.control===input,label.form===form];
    const result=[snapshot()];
    if(operation==='control-id')input.id='changed';
    if(operation==='class')input.className='changed';
    if(operation==='form-id')form.id='changed';
    if(operation==='form-attribute')input.setAttribute('form','f');
    if(operation==='reinsert')input.parentNode.append(input);
    if(operation==='remove-form')form.remove();
    result.push(snapshot());return result;
  });
}
export const parserRadioCases=['open-form','closed-form'];
export function measureParserRadios(cases) {
  return cases.map(kind=>{
    const doc=new DOMParser().parseFromString('<table><form id=f><tr><td><input type=radio name=g id=a checked></table>'+
      (kind==='closed-form'?'</form>':'')+'<input type=radio name=g id=b checked>','text/html');
    return ['a','b'].map(id=>{const input=doc.getElementById(id);return [input.checked,input.form?.id??null];});
  });
}
export const parserRadioMutationCases=['remove-form','move-form','move-a','move-cell','move-table','move-root',
  'form-attribute','form-attribute-foreign','form-id','a-name','b-name','a-default','b-default','clone','adopt'];
export function measureParserRadioMutations(cases) {
  return cases.map(operation=>{
    const doc=new DOMParser().parseFromString('<div id=root><table><form id=f><tr><td><input type=radio name=g id=a checked></table></form><input type=radio name=g id=b checked></div>','text/html');
    const a=doc.getElementById('a'),b=doc.getElementById('b'),form=doc.getElementById('f'),root=doc.getElementById('root');
    const snapshot=()=>[a.checked,b.checked,a.form===form,b.form===form];
    const results=[snapshot()];
    if(operation==='remove-form')form.remove();
    if(operation==='move-form')form.parentNode.append(form);
    if(operation==='move-a')a.parentNode.append(a);
    if(operation==='move-cell')a.parentNode.parentNode.append(a.parentNode);
    if(operation==='move-table')root.append(root.firstChild);
    if(operation==='move-root')doc.body.append(root);
    if(operation==='form-attribute')a.setAttribute('form','missing');
    if(operation==='form-attribute-foreign')a.setAttributeNS('urn:x','form','missing');
    if(operation==='form-id')form.id='changed';
    if(operation==='a-name')a.name='other';
    if(operation==='b-name')b.name='other';
    if(operation==='a-default'){a.defaultChecked=false;a.defaultChecked=true;}
    if(operation==='b-default'){b.defaultChecked=false;b.defaultChecked=true;}
    if(operation==='clone')doc.body.append(root.cloneNode(true));
    if(operation==='adopt'){document.adoptNode(root);document.body.append(root);}
    results.push(snapshot());root.remove();return results;
  });
}
