const sources=['undefined','null','false','true','0','-0','1','-1','2.9','-2.9','NaN','Infinity','-Infinity',
  '2147483647','2147483648','4294967295','4294967296','4294967297','9007199254740991','1e100','-1e100',
  '1n','Object(1n)','Symbol()','({})','({valueOf(){return 31}})',
  '({[Symbol.toPrimitive](hint){if(hint!=="number")throw Error(hint);return 17.9}})',
  '({valueOf(){throw new RangeError("conversion")}})',
  ...['','12','12px','0xff','1e2','\u00a011','\ud800'].map(JSON.stringify)];
const attributes=['','0','-0','+0','-1','+1',' \t\r\n\f12px','\v12','\u00a012','1.5','1e2','0xff',
  '+ 2','--2','01','2147483647','2147483648','4294967295','4294967296','9007199254740991',
  '000000000000000000000000000001','999999999999999999999999999','-0000000000000000000000000000','３','١'];
export const imageDimensionCases=['width','height','hspace','vspace'].flatMap(name=>[
  ...sources.map(source=>({name,source,raw:false})),...attributes.map(value=>({name,source:JSON.stringify(value),raw:true})),
]);
export function measureImageDimensions(cases){return cases.map(({name,source,raw})=>{
  const e=document.createElement('img');e.setAttribute(name,'before');
  const observer=new MutationObserver(()=>{});observer.observe(e,{attributes:true,attributeOldValue:true});let error=null;
  try{const value=Function('return ('+source+')')();if(raw)e.setAttribute(name,value);else e[name]=value;}catch(e){error=e.name}
  const records=observer.takeRecords().map(r=>[r.attributeName,r.attributeNamespace,r.oldValue]);observer.disconnect();
  return[error,e[name],e.getAttribute(name),records];
});}
export const imageFactoryCases=[[],...sources.map(source=>[source]),...sources.map(source=>['12',source]),['undefined','null'],['undefined','undefined'],['1','2','Symbol()']];
export function measureImageFactory(cases){return cases.map(sources=>{
  try{const args=sources.map(source=>Function('return ('+source+')')()),e=new Image(...args);
    return[null,e.width,e.height,e.getAttribute('width'),e.getAttribute('height'),e.ownerDocument===document,Object.getPrototypeOf(e)===HTMLImageElement.prototype];
  }catch(e){return[e.name]}
});}
