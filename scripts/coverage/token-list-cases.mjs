export const tokenListInitialValues = [null, '', 'a', 'a b c', ' a  b a\t c\n', 'A a', 'x\u00a0y z', '\ud800 \udc00'];
const calls = [
  ['item', []], ['contains', []], ['toggle', []], ['replace', []], ['replace', ['"a"']], ['supports', []],
  ...['undefined', 'null', 'true', '0', '1n', 'Symbol()', '""', '"a"', '"A"', '"a b"', '"\\t"', '"x\\u00a0y"', '"\\ud800"', '({[Symbol.toPrimitive](hint){return hint}})', '({toString(){throw new RangeError("conversion")}})'].flatMap(source =>
    ['contains', 'add', 'remove', 'toggle', 'supports', 'value', 'relList'].map(name => [name, [source]])),
  ...['undefined','null','-1','0','1','1.9','4294967296','NaN','Infinity','1n','Symbol()'].map(source=>['item',[source]]),
  ...['add','remove'].flatMap(name=>[[],['"a"','"b"'],['"a"','"a"'],['""','Symbol()'],['"a b"','""'],['""','({toString(){throw new RangeError("conversion")}})']].map(args=>[name,args])),
  ...['undefined','null','false','true','0','1','""','"false"','({valueOf(){throw Error("must not convert")}})'].flatMap(force=>['"a"','"z"'].map(token=>['toggle',[token,force]])),
  ...[['a','b'],['b','a'],['b','c'],['c','a'],['z','a'],['a','a'],['','a'],['a b',''],['a','a b']].map(args=>['replace',args.map(JSON.stringify)]),
];
export const tokenListCases = tokenListInitialValues.flatMap(initial => calls.map(([name,args]) => ({initial,name,args})));
export const relationTokens = ['alternate','author','bookmark','external','help','license','next','nofollow','noopener','noreferrer','opener','prev','search','tag','stylesheet','preload','modulepreload','prefetch','preconnect','dns-prefetch','prerender','icon','manifest','canonical','privacy-policy','terms-of-service','ar','payment','sponsored','ugc','compression-dictionary','expect','apple-touch-icon','apple-touch-icon-precomposed','apple-touch-startup-image','mask-icon','shortcut','serviceworker','subresource','me','', 'a b','NOOPENER','STYLESHEET','NoReFeRrEr','noopener\t'];
export function measureTokenLists(cases) {
  return cases.map(({initial,name,args}) => {
    const a = document.createElement('a'); if(initial!==null)a.setAttribute('rel',initial);
    const list = a.relList, observer = new MutationObserver(()=>{});
    observer.observe(a,{attributes:true,attributeOldValue:true});
    let result, error = null;
    try {
      const values=args.map(source=>(0,eval)(source));
      if(name==='value')list.value=values[0];
      else if(name==='relList')a.relList=values[0];
      else result=list[name](...values);
    } catch(e) { error=e.name; }
    const records=observer.takeRecords().map(r=>[r.attributeName,r.attributeNamespace,r.oldValue]);observer.disconnect();
    return {error,result:result===undefined?{type:'undefined'}:result,value:list.value,tokens:[...list],attribute:a.getAttribute('rel'),records};
  });
}
export function measureTokenSupports(tokens) {
  const sources=[['a','relList'],['area','relList'],['link','relList'],['link','sizes'],['output','htmlFor'],['video','controlsList'],['div','classList']];
  return sources.map(([tag,property])=>({tag,property,values:tokens.map(token=>{try{return document.createElement(tag)[property].supports(token)}catch(e){return e.name}})}));
}
export function measureTokenShape() {
  const C=DOMTokenList,p=C.prototype,list=document.createElement('a').relList;
  const descriptor=(object,key)=>{const d=Object.getOwnPropertyDescriptor(object,key);return{enumerable:d.enumerable,configurable:d.configurable,writable:d.writable,value:typeof d.value==='function'?[d.value.name,d.value.length,Object.hasOwn(d.value,'prototype')]:d.value,get:d.get?[d.get.name,d.get.length]:null,set:d.set?[d.set.name,d.set.length]:null}};
  const members=Object.getOwnPropertyNames(p).filter(n=>n!=='constructor').sort();
  const constructions=[()=>C(),()=>new C(),()=>new C(document.createElement('a')),()=>new(class extends C{})()].map(fn=>{try{fn();return'ok'}catch(e){return e.name}});
  const pd=Object.getOwnPropertyDescriptor(C,'prototype');
  return {constructorDescriptor:descriptor(p,'constructor'),prototypeDescriptor:[pd.value===p,pd.writable,pd.enumerable,pd.configurable],constructor:[C.name,C.length,Object.getPrototypeOf(C)===Function.prototype],prototype:[Object.getPrototypeOf(p)===Object.prototype,Object.getPrototypeOf(list)===p,list.constructor===C],tag:descriptor(p,Symbol.toStringTag),members:members.map(n=>[n,descriptor(p,n)]),iterator:descriptor(p,Symbol.iterator),iteratorIdentity:p.values===p[Symbol.iterator],arrayIdentity:['entries','keys','values','forEach'].map(n=>p[n]===Array.prototype[n]),constructions};
}
