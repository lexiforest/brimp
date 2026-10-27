export const inputIntegerFields = ['maxLength', 'minLength', 'size'];
const assignments = [
  'undefined', 'null', 'false', 'true', 'NaN', 'Infinity', '-Infinity', '-0',
  '-0.9', '0.9', '-1.9', '1.9', '1e100', '-1e100', 'Number.MAX_VALUE',
  'Number.MIN_VALUE', 'Number.MAX_SAFE_INTEGER', 'Number.MIN_SAFE_INTEGER',
  '1n', 'Object(1n)', 'Symbol()', 'Object(Symbol())', '[]', '[2]', '[1,2]',
  '{}', 'new Number(3.8)', '({valueOf(){return 9;}})',
  '({[Symbol.toPrimitive](hint){if(hint!=="number")throw Error("wrong hint");return 11;}})',
  '({valueOf(){return {};},toString(){return "12";}})',
  '({valueOf(){throw new RangeError("conversion");}})',
  ...[-4294967297,-4294967296,-4294967295,-2147483649,-2147483648,-2147483647,-1,0,1,2,
    2147483646,2147483647,2147483648,2147483649,4294967294,4294967295,4294967296,4294967297,
    8589934591,8589934592,8589934593].map(String),
  ...['', ' ', '\u00a0', '\u000b2', '2.9', '-2.9', '0x10', '0b11', '0o10', '2e2', '2junk', 'Infinity'].map(JSON.stringify),
];
const attributes = [
  '', ' ', '\t\n\f\r ', '0', '-0', '+0', '00', '00000000001', '+1', '-1', '-01',
  '1.9', '-0.9', '+0.9', '.9', '1e2', '1e999', '0x10', '0b11', '2junk', '2 3',
  '++2', '--2', '+-2', '-+2', '+ 2', '- 2', 'NaN', 'Infinity', '٢', '２',
  '2147483646', '2147483647', '2147483648', '2147483649', '4294967295', '4294967296',
  '-2147483648', '-2147483649', '9007199254740991', '9'.repeat(100), '0'.repeat(100)+'7',
  ...['\t','\n','\f','\r',' ','\u000b','\u00a0','\u2000','\ufeff','\0','\ud800'].flatMap(prefix=>[prefix+'2',prefix+'-0']),
];
export const inputIntegerCases = inputIntegerFields.flatMap(property => [
  ...assignments.map(source => ({property, kind:'property', source})),
  ...attributes.map(source => ({property, kind:'attribute', source})),
]);

export function measureInputIntegers(cases) {
  return cases.map(({property,kind,source}) => {
    const input=document.createElement('input'), attribute=property.toLowerCase();
    input[property]=7;
    const observer=new MutationObserver(()=>{});
    observer.observe(input,{attributes:true,attributeOldValue:true,attributeFilter:[attribute]});
    let error=null;
    try {
      if(kind==='attribute')input.setAttribute(attribute,source);
      else input[property]=Function('return ('+source+')')();
    } catch(e) { error={name:e.name,code:e.code??null}; }
    const records=observer.takeRecords().map(record=>[record.attributeName,record.attributeNamespace,record.oldValue]);
    observer.disconnect();
    return {error,value:input[property],attribute:input.getAttribute(attribute),records};
  });
}

export const inputLengthConstraintCases = [['input','text'],['input','number'],['input','file'],['input','hidden'],['textarea',null]]
  .flatMap(([tag,type])=>['minLength','maxLength'].flatMap(property=>[null,'0','5','10','-1','bad','2147483648','5junk']
    .flatMap(other=>[0,5,10].map(value=>({tag,type,property,other,value})))));
export function measureLengthConstraints(cases) {
  return cases.map(({tag,type,property,other,value})=>{
    const input=document.createElement(tag);if(type!==null)input.type=type;
    const peer=property==='minLength'?'maxlength':'minlength';
    if(other!==null)input.setAttribute(peer,other);
    const observer=new MutationObserver(()=>{});observer.observe(input,{attributes:true});
    let error=null;try{input[property]=value;}catch(e){error=e.name;}
    const records=observer.takeRecords().length;observer.disconnect();
    return {error,value:input[property],attribute:input.getAttribute(property.toLowerCase()),records};
  });
}
