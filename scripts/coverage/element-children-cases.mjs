const patterns=[[],['text'],['comment'],['text','comment'],['html'],['text','html','comment'],['svg','text','foreign','html'],['comment','prefixed','upper','comment'],['html','html','svg','foreign']];
export const elementChildrenCases=['document','html','svg','foreign','template','fragment','template-content','shadow'].flatMap(parent=>patterns.map(pattern=>({parent,pattern})));
export function measureElementChildren(cases){
  const H='http://www.w3.org/1999/xhtml',S='http://www.w3.org/2000/svg';
  return cases.map(c=>{
    const d=new Document(),labels=new Map();let next=0;
    const make=kind=>{
      const node=kind==='text'?d.createTextNode('text'):kind==='comment'?d.createComment('comment'):d.createElementNS(kind==='svg'?S:kind==='foreign'?'urn:foreign':H,kind==='svg'?'circle':kind==='prefixed'?'p:div':kind==='upper'?'DIV':'div');
      labels.set(node,kind+':'+next++);return node;
    };
    let parent;
    if(c.parent==='document')parent=d;
    else if(c.parent==='fragment')parent=d.createDocumentFragment();
    else if(c.parent==='template-content')parent=d.createElementNS(H,'template').content;
    else if(c.parent==='shadow')parent=d.createElementNS(H,'div').attachShadow({mode:'open'});
    else parent=d.createElementNS(c.parent==='svg'?S:c.parent==='foreign'?'urn:foreign':H,c.parent==='template'?'template':'root');
    const list=parent.children,out=[];
    const label=n=>n===null?null:labels.get(n);
    const read=()=>out.push({members:[...list].map(label),count:parent.childElementCount,first:label(parent.firstElementChild),last:label(parent.lastElementChild),same:list===parent.children});
    read();
    for(const kind of c.pattern){if(c.parent==='document'&&(kind==='text'||kind!=='comment'&&parent.documentElement))continue;parent.appendChild(make(kind));read()}
    const first=parent.firstElementChild;if(first){first.remove();read();parent.appendChild(first);read()}
    const extra=make('svg');if(c.parent==='document'&&parent.documentElement)parent.replaceChild(extra,parent.documentElement);else parent.insertBefore(extra,parent.firstChild);read();
    extra.remove();read();
    parent.replaceChildren(make('html'));read();
    parent.replaceChildren();read();
    return out;
  });
}
