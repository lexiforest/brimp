const roots=['standard','empty-html','empty-xml','html','prefix-html','upper-html','div','svg','xml','body','frameset'];
const layouts=['empty','direct','nested','foreign','upper','prefixed','frameset-first','body-first','template','shadow'];
export const documentStructureReadCases=roots.flatMap(root=>layouts.map(layout=>({root,layout})));
const incoming=['body','frameset','prefix-body','upper-body','div','svg-body','null-body','null','undefined','false','0','string','object','text','fragment','document','current','nested','sibling','ancestor','donor','root'];
export const documentBodyWriteCases=roots.flatMap(root=>['empty','direct','nested','frameset-first','body-first'].flatMap(layout=>incoming.map(incoming=>({root,layout,incoming}))));

export function measureDocumentStructure(cases,write=false){
  const H='http://www.w3.org/1999/xhtml',S='http://www.w3.org/2000/svg';
  return cases.map(c=>{
    const d=['standard','empty-html'].includes(c.root)?document.implementation.createHTMLDocument():new Document();
    if(c.root==='empty-html')d.documentElement.remove();
    const roots={html:[H,'html'],'prefix-html':[H,'p:html'],'upper-html':[H,'HTML'],div:[H,'div'],svg:[S,'svg'],xml:[null,'root'],body:[H,'body'],frameset:[H,'frameset']};
    if(roots[c.root])d.appendChild(d.createElementNS(...roots[c.root]));
    const root=d.documentElement,labels=new Map([[d,'document']]);let count=0;
    const label=n=>n===null?null:labels.get(n)??(labels.set(n,`${n.localName??n.nodeName}:${++count}`),labels.get(n));
    const snap=n=>[label(n),n.nodeType,n.localName??null,n.namespaceURI??null,...[...n.childNodes].map(snap)];
    if(root){
      root.replaceChildren();labels.set(root,'root');let parent=root;
      if(c.layout==='nested'){parent=d.createElementNS(H,'div');root.appendChild(parent)}
      if(c.layout==='template'){const t=d.createElementNS(H,'template');root.appendChild(t);parent=t.content}
      if(c.layout==='shadow'){const host=d.createElementNS(H,'div');root.appendChild(host);parent=host.attachShadow({mode:'open'})}
      if(c.layout!=='empty'){
        const names=c.layout==='frameset-first'?['head','frameset','body']:c.layout==='body-first'?['head','body','frameset']:['head','body'];
        for(const name of names){const n=d.createElementNS(c.layout==='foreign'?S:H,c.layout==='upper'?name.toUpperCase():c.layout==='prefixed'?'p:'+name:name);labels.set(n,name);parent.appendChild(n);n.appendChild(d.createTextNode(name));}
      }
    }
    const read=()=>[label(d.documentElement),label(d.head),label(d.body)];
    const before=read();snap(d);
    if(!write)return {before,tree:snap(d)};
    const donor=document.implementation.createHTMLDocument(),body=d.body;
    let value;
    switch(c.incoming){
      case'body':case'frameset':case'div':value=d.createElementNS(H,c.incoming);break;
      case'prefix-body':value=d.createElementNS(H,'p:body');break;
      case'upper-body':value=d.createElementNS(H,'BODY');break;
      case'svg-body':value=d.createElementNS(S,'body');break;
      case'null-body':value=d.createElementNS(null,'body');break;
      case'null':value=null;break;case'undefined':value=undefined;break;case'false':value=false;break;case'0':value=0;break;case'string':value='body';break;case'object':value={};break;
      case'text':value=d.createTextNode('body');break;case'fragment':value=d.createDocumentFragment();break;case'document':value=d;break;
      case'current':value=body;break;
      case'root':value=root;break;
      case'nested':value=d.createElementNS(H,'body');(body??root)?.appendChild(value);break;
      case'sibling':value=d.createElementNS(H,'body');root?.appendChild(value);break;
      case'ancestor':value=d.createElementNS(H,'body');if(root){const old=[...root.childNodes];root.appendChild(value);for(const n of old)value.appendChild(n)}break;
      case'donor':value=donor.body;break;
    }
    if(value instanceof Node&&!labels.has(value))labels.set(value,'incoming');
    labels.set(donor,'donor');labels.set(donor.documentElement,'donor-root');labels.set(donor.head,'donor-head');if(donor.body!==value)labels.set(donor.body,'donor-body');
    snap(d);snap(donor);if(value instanceof Node)snap(value);
    const observer=new MutationObserver(()=>{});observer.observe(d,{childList:true,subtree:true});observer.observe(donor,{childList:true,subtree:true});
    let error=null;try{d.body=value}catch(e){error=e.name}
    const records=observer.takeRecords().map(r=>[r.type,label(r.target),[...r.addedNodes].map(label),[...r.removedNodes].map(label),label(r.previousSibling),label(r.nextSibling)]);observer.disconnect();
    return {before,after:read(),error,tree:snap(d),donor:snap(donor),incoming:value instanceof Node?[label(value.parentNode),value.ownerDocument===d]:null,records};
  });
}
