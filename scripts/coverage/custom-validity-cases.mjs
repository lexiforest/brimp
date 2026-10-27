const tags=['input','button','select','textarea','fieldset','output','object'];
const values=['missing','undefined','null','empty','text','whitespace','linebreaks','nul','surrogate','false','number','NaN','bigint','symbol','object','throw','mutate'];
export const customValidityCases=tags.flatMap(tag=>['plain','disabled','readonly','datalist','barred-type'].flatMap(state=>values.map(kind=>({tag,state,kind}))));
export function measureCustomValidity(cases){
  return cases.map(({tag,state,kind})=>{
    const control=document.createElement(tag),parent=document.createElement(state==='datalist'?'datalist':'div');parent.append(control);
    if(state==='disabled')control.setAttribute('disabled','');if(state==='readonly')control.setAttribute('readonly','');
    if(state==='barred-type')control.setAttribute('type',tag==='button'?'reset':'hidden');
    control.setCustomValidity('before');
    let value;const conversions=[];
    if(kind==='null')value=null;if(kind==='empty')value='';if(kind==='text')value='custom message';
    if(kind==='whitespace')value=' \t\n ';if(kind==='linebreaks')value='a\r\nb\rc\nd';
    if(kind==='nul')value='a\0b';if(kind==='surrogate')value='a\ud800b\udc00';
    if(kind==='false')value=false;if(kind==='number')value=123;if(kind==='NaN')value=NaN;if(kind==='bigint')value=123n;if(kind==='symbol')value=Symbol('message');
    if(kind==='object')value={[Symbol.toPrimitive](hint){conversions.push(hint);return 'converted';}};
    if(kind==='throw')value={[Symbol.toPrimitive](){conversions.push('throw');throw new RangeError('conversion');}};
    if(kind==='mutate')value={toString(){conversions.push('mutate');control.setAttribute('disabled','');return 'changed';}};
    let error=null,result;try{result=kind==='missing'?control.setCustomValidity():control.setCustomValidity(value);}catch(e){error=e.name;}
    const invalid=[];control.addEventListener('invalid',e=>invalid.push([e.bubbles,e.cancelable,e.composed,e.isTrusted]));
    return [error,typeof result,conversions,control.validity.customError,control.validity.valid,control.validationMessage,control.checkValidity(),invalid];
  });
}
