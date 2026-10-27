const types=['text','number','range','date','time','datetime-local','month','week','file'];
const numbers=['-Infinity','Infinity','NaN','-1e100','1e100','-8640000000000001','-8640000000000000','-62135596800001','-62135596800000','-86400001','-86400000','-1.9','-1','-0.9','-0.1','-0','0','0.1','0.9','1','1.9','59999.9','60000','86399999.9','86400000','86400001','951868800001','8640000000000000','8640000000000001','0.30000000000000004','1e-7','1e21','5e-324'];
numbers.push('0.5','-0.5','10','100','1000','1100','1230','59999.5','86399999.5');
for(const base of [-62135596800000,946684800000,1e14,1e15,8640000000000000])for(const delta of [-1,-0.75,-0.5,-0.25,0.25,0.5,0.75,1]){const text=String(base+delta);if(!numbers.includes(text))numbers.push(text);}
const conversions=['null','undefined','true','false','string-number','string-date','bigint','symbol','object','boxed-number','valueOf','date-invalid','date-zero','date-negative','date-overridden','proxy-date'];
export const inputValueConversionCases=types.flatMap(type=>['valueAsNumber','valueAsDate'].flatMap(property=>
  [...numbers.map(number=>({kind:'number',number})),...conversions.map(kind=>({kind}))].map(value=>({type,property,...value}))));
export function measureInputValueConversions(cases) {
  return cases.map(({type,property,kind,number})=>{
    const input=document.createElement('input');input.type=type;
    if(type==='range'){input.min='-100';input.max='100';input.step='0.1';}
    input.defaultValue=type==='file'?'':type==='date'?'2000-01-02':type==='time'?'12:34:56.789':type==='datetime-local'?'2000-01-02T12:34:56.789':'17';
    const log=[];let value;
    if(kind==='number')value=Number(number);
    if(kind==='null')value=null;if(kind==='true')value=true;if(kind==='false')value=false;
    if(kind==='string-number')value='123';if(kind==='string-date')value='2000-01-01';
    if(kind==='bigint')value=1n;if(kind==='symbol')value=Symbol();if(kind==='object')value={};
    if(kind==='boxed-number')value=new Number(123);
    if(kind==='valueOf')value={valueOf(){log.push('valueOf');return 123;},toString(){log.push('toString');return '456';}};
    if(kind.startsWith('date-')||kind==='proxy-date'){
      value=new Date(kind==='date-invalid'?NaN:kind==='date-negative'?-1:0);
      if(kind==='date-overridden'){value.valueOf=()=>{log.push('valueOf');return 123;};value.getTime=()=>{throw Error('getTime override');};}
      if(kind==='proxy-date')value=new Proxy(value,{});
    }
    let error=null;try{input[property]=value;}catch(e){error=e.name;}
    const date=input.valueAsDate,num=input.valueAsNumber;
    const result=[error,input.value,Number.isNaN(num)?'NaN':Object.is(num,-0)?'-0':String(num),date===null?null:date.getTime(),log];
    input.defaultValue=type==='file'?'':type==='date'?'2001-01-01':type==='time'?'01:02':type==='datetime-local'?'2001-01-01T01:02':'19';
    result.push(input.value);return result;
  });
}

export const inputTimeSerializationCases=['time','datetime-local'].flatMap(type=>
  ['', '60','120','1','2','0.001','0.1','0.0001','any','ANY','invalid','0','-1',' 1','+1','1.0000000000000001','1e100','1e-100'].flatMap(step=>
    ['valueAsNumber','valueAsDate'].flatMap(property=>[0,1,10,100,1000,1100,1230,60000].map(number=>({type,step,property,number})))));
export function measureInputTimeSerialization(cases) {
  return cases.map(({type,step,property,number})=>{
    const input=document.createElement('input');input.type=type;if(step!=='')input.setAttribute('step',step);
    let error=null;try{input[property]=number;}catch(e){error=e.name;}
    return [error,input.value];
  });
}
export const inputConversionGetterCases=['date','time','datetime-local','number','range','text','month','week'].flatMap(type=>
  ['', '0001-01-01','0099-12-31','000099-12-31','1900-02-29','2000-02-29','275760-09-13',
    '00:00','01:02:01.001','23:59:59.999','00:00:00.1','00:00:00.010',
    '2000-02-29T23:59:59.999','275760-09-13T00:00','-0','1.0000000000000002','1e-100','1e100'].map(value=>({type,value})));
export function measureInputConversionGetters(cases) {
  return cases.map(({type,value})=>{
    const i=document.createElement('input');i.type=type;i.value=value;const n=i.valueAsNumber,d=i.valueAsDate;
    return [i.value,Number.isNaN(n)?'NaN':Object.is(n,-0)?'-0':String(n),d===null?null:d.getTime()];
  });
}
