// These cases are measured in installed Safari; expected values are not inferred
// from the implementation. Keep parser, constraint and transition paths shared.
export const inputSanitizationCases = [];
function add(type, values, attributes = {}) {
  for (const value of values) inputSanitizationCases.push({type, attributes, value});
}
const dates = ['', '2024-02-29', '2023-02-29', '1900-02-29', '2000-02-29',
  '0001-01-01', '0000-01-01', '1-01-01', '00001-01-01', '0'.repeat(50)+'1-01-01',
  '275760-09-12', '275760-09-13', '275760-09-14', '275760-10-01', '275761-01-01',
  '9'.repeat(50)+'-01-01', '2024-04-31', '2024-04-30', '2024-00-01', '2024-13-01',
  '2024-01-00', '2024-01-32', '2024-1-01', '2024-01-1', '+2024-01-01',
  ' 2024-01-01', '2024-01-01 ', '2024-01-01\n', '2024-01-01\r', '2024-01-01Z'];
const times = ['', '00:00', '23:59', '01:02:00', '01:02:03', '01:02:03.1', '01:02:03.10',
  '01:02:03.100', '01:02:03.010', '01:02:03.001', '01:02:03.000', '01:02:03.1234',
  '01:02:03.', '01:02.1', '24:00', '23:59:60', '1:02', '01:2', '01:02:3',
  ' 01:02', '01:02 ', '01:02\n', '01:02\r', '01:02:03Z'];
add('date', dates);
add('time', times);
add('datetime-local', [...dates.map(d=>d+'T00:00'), ...times.map(t=>'2024-02-29T'+t),
  '00001-01-01T01:02:00.000', '2024-02-29 01:02:03.010', '2024-02-29t01:02',
  '2024-02-29  01:02', '275760-09-13T00:00:00.001', '275760-09-13T00:01',
  '275760-09-13T00:00:00.000']);
const numbers = ['', '01', '+1', '1.', '-0', '1e+2', '.5', '1e309', '1e-999',
  ' 1', '1 ', '1\n', '1\r', '0x10', 'Infinity', 'NaN', '-.5'];
add('number', numbers);
for (const type of ['date','time','datetime-local','number']) {
  add(type, [type==='date'?'2024-02-29':type==='time'?'01:02:03.100':type==='number'?'3.5':'2024-02-29T01:02:03.100'],
    {min:'9999-01-01',max:'0001-01-01',step:'7'});
}
const ranges = [{}, {min:'0',max:'10',step:'3'}, {min:'10',max:'0'}, {min:'-10',max:'10',step:'4'},
  {min:'.1',max:'.9',step:'.2'}, {min:'0',max:'.5'}, {step:'any'}, {step:'2',value:'0.5'},
  {step:'2',value:'5'}, {min:'abc',max:'bad',step:'ANY'}, {min:'1',max:'2',step:'10',value:'5'},
  {min:'-1e308',max:'1e308',step:'any'}, {min:'1e20',max:'1e21',step:'any'},
  {min:'-5',max:'-1',step:'any'}, {min:'0.1',max:'1',step:'0.2'}, {min:'0',step:'0'},
  {max:'10',step:'-1'}, {step:'1e-999'}, {min:'-1',max:'1',step:'1e-20'}];
for(const attributes of ranges) add('range', [...numbers,'-100','2','5','10','100',
  '0.1234567890123456789','0.30000000000000004','1e-7','1e-20','1e308','1e20'],attributes);

// Decimal arithmetic and serialization boundaries are observable even though
// the engine's ordinary Number type uses binary floating-point arithmetic.
for(const attributes of [
  {min:'-1000',max:'1000',step:'any'}, {min:'-1000',max:'1000',step:'0.3'},
  {min:'0.00000000000000001',max:'0.0000000000000001',step:'0.00000000000000003'},
  {min:'1.00000000000000001',max:'1.00000000000000009',step:'0.00000000000000002'},
  {min:'-1e308',max:'1e308',step:'1e-1023'},
]) add('range',['-0','0e99999','0e-99999','1e-1023','1e-1024','0.0000000000000000001',
  '0.9999999999999999999','123.4567890123456789','10e1','1.00e2','-0.15','0.15','0.45',
  '1.00000000000000005','1.00000000000000008'],attributes);

export function measureInputSanitization(cases) {
  return cases.map(({type, attributes, value}) => {
    const rows=[];
    for(const path of ['setter','attribute','transition','reset']) {
      const i=document.createElement('input');
      for(const [name,v] of Object.entries(attributes))i.setAttribute(name,v);
      if(path==='transition'){i.value=value;i.type=type;}
      else {
        i.type=type;
        if(path==='setter')i.value=value;
        else {i.defaultValue=value;
          if(path==='reset'){const form=document.createElement('form');form.appendChild(i);i.value='99';form.reset();}
        }
      }
      const sanitized=i.value;i.type='text';rows.push([sanitized,i.value]);
    }
    return rows;
  });
}
