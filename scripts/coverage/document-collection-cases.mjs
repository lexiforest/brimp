export const documentCollectionNames=['forms','images','embeds','plugins','links','scripts','anchors','applets'];
export const documentCollectionCases=['html','xml','svg'].flatMap(document=>['form','img','embed','object','applet','a','area','script'].flatMap(tag=>['html','prefixed','upper','svg','null'].flatMap(namespace=>['none','empty','value','namespaced','upper'].flatMap(attributes=>['direct','nested','template','shadow','detached'].map(placement=>({document,tag,namespace,attributes,placement}))))));

export function measureDocumentCollections(cases){
  const names=['forms','images','embeds','plugins','links','scripts','anchors','applets'];
  const H='http://www.w3.org/1999/xhtml',S='http://www.w3.org/2000/svg';
  return cases.map(c=>{
    const d=c.document==='html'?document.implementation.createHTMLDocument():new Document();
    if(c.document!=='html')d.appendChild(d.createElementNS(c.document==='svg'?S:null,c.document==='svg'?'svg':'root'));
    const collections=names.map(name=>d[name]);
    const element=d.createElementNS(['html','prefixed','upper'].includes(c.namespace)?H:c.namespace==='svg'?S:null,c.namespace==='prefixed'?'p:'+c.tag:c.namespace==='upper'?c.tag.toUpperCase():c.tag);
    if(c.attributes!=='none')for(const name of ['href','name'])element.setAttributeNS(c.attributes==='namespaced'?'urn:collection':null,c.attributes==='namespaced'?'p:'+name:c.attributes==='upper'?name.toUpperCase():name,c.attributes==='empty'?'':'match');
    if(c.tag==='object')element.setAttribute('type','application/x-java-applet');
    let parent=d.body??d.documentElement;
    if(c.placement==='nested'){const box=d.createElementNS(H,'div');parent.appendChild(box);parent=box}
    if(c.placement==='template'){const t=d.createElementNS(H,'template');parent.appendChild(t);parent=t.content}
    if(c.placement==='shadow'){const host=d.createElementNS(H,'div');parent.appendChild(host);parent=host.attachShadow({mode:'open'})}
    if(c.placement!=='detached')parent.appendChild(element);
    const membership=collections.map((collection,i)=>[collection.length,collection.item(0)===element,collection[0]===element,collection===d[names[i]]]);
    const aliases=d.plugins===d.embeds;
    element.remove();
    return {membership,aliases,removed:collections.map(collection=>collection.length)};
  });
}
