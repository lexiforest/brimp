import {inputValidityCases} from './input-validity-cases.mjs';
export const inputValidationCases=[...inputValidityCases];
export function measureInputValidation(cases){
  return cases.map(({type,value,attributes,custom})=>{
    const input=document.createElement('input');input.type=type;
    for(const [name,v] of Object.entries(attributes))input.setAttribute(name,v);
    input.value=value;if(custom!==undefined)input.setCustomValidity(custom);
    const events=[];input.addEventListener('invalid',e=>events.push([e.isTrusted,e.bubbles,e.cancelable,e.composed]));
    return [input.validationMessage,input.checkValidity(),events];
  });
}

for(const type of ['text','email','url','number','date','time','datetime-local','checkbox','radio','file'])
  for(const attributes of [{required:'',name:'g'},{required:'',name:'g',multiple:''},{pattern:'x',title:'A helpful title'},{pattern:'x',title:' \ncustom\t'},{min:'2',max:'1',step:'3'}])
    for(const custom of ['', 'custom message'])inputValidationCases.push({type,value:type==='file'?'':'bad',attributes,custom});
for(const type of ['number','range'])for(const bound of ['-0','01.00','1e3','1e-7','1e21','1.2345678901234567','1.e2','+2','-5e-324'])
  for(const value of ['-1e22','1e22'])for(const constraint of ['min','max'])inputValidationCases.push({type,value,attributes:{[constraint]:bound}});
for(const [type,bounds,values] of [
  ['date',['02000-01-01','0001-01-01','275760-09-13'],['0001-01-01','2000-01-01','275760-09-13']],
  ['time',['00:00','12:00:00.010','23:59:59.999'],['00:00','11:00','23:59:59.999']],
  ['datetime-local',['02000-01-01 12:00:00.010','0001-01-01T00:00','275760-09-13T00:00'],['0001-01-01T00:00','2000-01-01T12:00','275760-09-13T00:00']],
])for(const bound of bounds)for(const value of values)for(const constraint of ['min','max'])for(const step of ['any','0.001','1','60','1e-100'])
  inputValidationCases.push({type,value,attributes:{[constraint]:bound,step}});
for(const title of ['',' ','\v title\v','  multiple \n\t words  ','\u00a0title\u00a0','line\r\nbreak','a\0b','a\ud800b'])
  inputValidationCases.push({type:'text',value:'mismatch',attributes:{pattern:'x',title}});

export async function measureInputValidationEditing(page,cases){
  const results=[];
  for(const {type,text} of cases){
    await page.evaluate(`(()=>{document.body.innerHTML='<input id="edit" minlength="3" pattern="x" required>';document.getElementById('edit').type=${JSON.stringify(type)};return null})()`);
    await page.typeText('#edit',text);
    results.push(JSON.parse(await page.evaluate(`JSON.stringify((()=>{
      const i=document.getElementById('edit'),events=[];
      i.addEventListener('invalid',e=>events.push([e.isTrusted,e.bubbles,e.cancelable,e.composed]));
      const snapshot=()=>{events.length=0;return [i.validationMessage,i.checkValidity(),events.slice()]};
      const states=[snapshot()];i.removeAttribute('pattern');states.push(snapshot());
      i.setAttribute('maxlength','2');states.push(snapshot());
      i.setAttribute('maxlength','1');states.push(snapshot());
      i.setCustomValidity('custom');states.push(snapshot());i.disabled=true;states.push(snapshot());
      return states;
    })())`)));
  }
  return results;
}
