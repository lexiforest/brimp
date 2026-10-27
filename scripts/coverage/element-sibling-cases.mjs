const kinds=['html','svg','foreign','text','comment'];
const patterns=[[],...kinds.map(kind=>[kind]),...kinds.flatMap(a=>kinds.map(b=>[a,b])),...kinds.map(kind=>['text','comment',kind,'comment','svg'])];
export const elementSiblingCases=['document','html','svg','foreign','fragment','template','template-content','shadow'].flatMap(parent=>patterns.map(pattern=>({parent,pattern})));
export function measureElementSiblings(cases){
  const H='http://www.w3.org/1999/xhtml',S='http://www.w3.org/2000/svg';
  return cases.map(c=>{
    const d=new Document(),labels=new Map(),nodes=[];let number=0;
    const make=kind=>{const n=kind==='text'?d.createTextNode('text'):kind==='comment'?d.createComment('comment'):d.createElementNS(kind==='svg'?S:kind==='foreign'?'urn:foreign':H,kind==='svg'?'circle':'div');labels.set(n,kind+':'+number++);nodes.push(n);return n};
    let parent;
    if(c.parent==='document')parent=d;
    else if(c.parent==='fragment')parent=d.createDocumentFragment();
    else if(c.parent==='template-content')parent=d.createElementNS(H,'template').content;
    else if(c.parent==='shadow')parent=d.createElementNS(H,'div').attachShadow({mode:'open'});
    else parent=d.createElementNS(c.parent==='svg'?S:c.parent==='foreign'?'urn:foreign':H,c.parent==='template'?'template':'root');
    for(const kind of c.pattern){if(c.parent==='document'&&(kind==='text'||kind!=='comment'&&d.documentElement))continue;parent.appendChild(make(kind))}
    const label=n=>n===null?null:labels.get(n),out=[];
    const read=()=>out.push(nodes.map(node=>[label(node),label(node.previousElementSibling),label(node.nextElementSibling)]));
    read();
    const first=parent.firstChild;if(first){first.remove();read();parent.appendChild(first);read()}
    const middle=parent.childNodes[1];if(middle){middle.remove();read()}
    const moved=parent.firstChild;
    if(moved){const other=document.implementation.createHTMLDocument(),box=other.createDocumentFragment();box.appendChild(make('html'));box.appendChild(other.adoptNode(moved));box.appendChild(make('svg'));read();box.replaceChildren();read()}
    parent.replaceChildren();read();return out;
  });
}
