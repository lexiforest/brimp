export const anchorStringFields=['charset','coords','download','hreflang','name','ping','rel','rev','shape','target','type'];
const sources=['undefined','null','false','true','0','1n','Symbol()','({})','({[Symbol.toPrimitive](hint){if(hint!=="string")throw Error("hint");return "converted"}})','({toString(){throw new RangeError("conversion")}})',...['',' Plain VALUE ','\0','a\ud800b\udc00','😀','a\r\nb'].map(JSON.stringify)];
export const anchorAttributeCases=[...anchorStringFields,'text','referrerPolicy'].flatMap(name=>sources.map(source=>({name,source})));
export function measureAnchorAttributes(cases){return cases.map(({name,source})=>{
 const a=document.createElement('a');a.innerHTML='<b>before</b>text<!--comment-->';a.setAttribute(name.toLowerCase(),'before');
 const observer=new MutationObserver(()=>{});observer.observe(a,{attributes:true,attributeOldValue:true,childList:true,subtree:true});let error=null;
 try{a[name]=Function('return ('+source+')')()}catch(e){error=e.name}
 const records=observer.takeRecords().map(r=>[r.type,r.attributeName,r.attributeNamespace,r.oldValue,r.addedNodes.length,r.removedNodes.length]);observer.disconnect();
 return [error,a[name],a.getAttribute(name.toLowerCase()),a.textContent,records];
});}
