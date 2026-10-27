export const temporalValidationActions={
  none:'',sameValue:'i.value=""',custom:'i.setCustomValidity("")',required:'i.required=true;i.required=false',
  disabled:'i.disabled=true;i.disabled=false',readonly:'i.readOnly=true;i.readOnly=false',min:'i.min=""',max:'i.max=""',step:'i.step=""',
  pattern:'i.pattern=""',title:'i.title=""',name:'i.name=""',valueAttr:'i.defaultValue=""',typeSame:'i.type=type',
  typeRoundtrip:'i.type="text";i.type=type',maxlength:'i.setAttribute("maxlength","3")',minlength:'i.setAttribute("minlength","3")',
  multiple:'i.multiple=true',clone:'i=i.cloneNode()',adopt:'document.implementation.createHTMLDocument().adoptNode(i)',
  move:'document.body.append(i);i.remove()',reset:'form.reset()',
};
export const temporalValidationCases=['date','time','datetime-local'].flatMap(type=>['attribute','script','parser'].flatMap(setup=>
  ['empty','invalid','reset','default'].flatMap(clear=>Object.keys(temporalValidationActions).map(action=>({type,setup,clear,action})))));
export function measureTemporalValidation(cases,actions){
  return cases.map(({type,setup,clear,action})=>{
    const good=type==='date'?'2000-01-01':type==='time'?'12:00':'2000-01-01T12:00',form=document.createElement('form');let i;
    if(setup==='parser'){form.innerHTML='<input type="'+type+'" value="'+good+'">';i=form.firstChild;}
    else{i=document.createElement('input');i.type=type;form.append(i);if(setup==='attribute')i.defaultValue=good;else i.value=good;}
    if(clear==='empty')i.value='';if(clear==='invalid')i.value='invalid';if(clear==='reset')form.reset();if(clear==='default')i.defaultValue='';
    const before=[i.value,i.validity.valid,i.checkValidity()];
    // Keep actions as page code so cloning can replace the local control.
    eval(actions[action]);
    const after=[i.value,i.validity.valid,i.checkValidity()];form.remove();i.remove();return [before,after];
  });
}
