export const elementConstructionCases = ['HTMLElement','HTMLInputElement','HTMLButtonElement','HTMLAnchorElement'].flatMap(base =>
  ['undefined','null','1','Symbol()','({extends:"input"})','({extends:"x-unknown"})','({get extends(){throw new RangeError("ignored option")}})'].map(options=>({base,options})));
export function measureElementConstruction(cases) {
  return cases.map(({base,options},index)=>{
    const name='construction-case-'+index,C=class extends globalThis[base]{};
    let registered=null;try{customElements.define(name,C,(0,eval)(options))}catch(e){registered=e.name}
    const make=fn=>{try{const e=fn();return {tag:e.localName,is:e.getAttribute('is'),custom:e instanceof C,unknown:e instanceof HTMLUnknownElement,owner:e.ownerDocument===document}}catch(e){return {error:e.name}}};
    const direct=make(()=>new C()),explicit=make(()=>Reflect.construct(HTMLElement,[],C));
    let reads=0;
    const created=make(()=>document.createElement(name,{get is(){reads++;throw Error('ignored')}}));
    const builtin=make(()=>document.createElement('input',{is:name}));
    const root=document.createElement('div');root.innerHTML='<input is="'+name+'"><'+name+'></'+name+'>';
    customElements.upgrade(root);
    const upgraded=[root.children[0],root.children[1]].map(e=>[e.localName,e instanceof C,e.getAttribute('is')]);
    return {registered,direct,explicit,created,builtin,reads,upgraded};
  });
}

export const builtinElementInterfaces = ["HTMLUnknownElement", "HTMLAnchorElement", "HTMLBaseElement", "HTMLPictureElement", "HTMLImageElement", "HTMLIFrameElement", "HTMLEmbedElement", "HTMLObjectElement", "HTMLParamElement", "HTMLMediaElement", "HTMLVideoElement", "HTMLAudioElement", "HTMLSourceElement", "HTMLTrackElement", "HTMLCanvasElement", "HTMLMapElement", "HTMLAreaElement", "HTMLFormElement", "HTMLFieldSetElement", "HTMLLegendElement", "HTMLLabelElement", "HTMLInputElement", "HTMLButtonElement", "HTMLSelectElement", "HTMLDataListElement", "HTMLOptGroupElement", "HTMLOptionElement", "HTMLTextAreaElement", "HTMLOutputElement", "HTMLProgressElement", "HTMLMeterElement", "HTMLTableElement", "HTMLTableCaptionElement", "HTMLTableColElement", "HTMLTableSectionElement", "HTMLTableRowElement", "HTMLTableCellElement", "HTMLHeadElement", "HTMLTitleElement", "HTMLLinkElement", "HTMLMetaElement", "HTMLStyleElement", "HTMLParagraphElement", "HTMLHRElement", "HTMLPreElement", "HTMLQuoteElement", "HTMLOListElement", "HTMLUListElement", "HTMLLIElement", "HTMLDListElement", "HTMLDivElement", "HTMLSpanElement", "HTMLDataElement", "HTMLTimeElement", "HTMLBRElement", "HTMLBodyElement", "HTMLHeadingElement", "HTMLHtmlElement", "HTMLScriptElement", "HTMLTemplateElement", "HTMLSlotElement", "HTMLModElement", "HTMLDetailsElement", "HTMLMenuElement", "HTMLDialogElement", "HTMLMarqueeElement", "HTMLFrameSetElement", "HTMLFrameElement", "HTMLDirectoryElement", "HTMLFontElement"];
export function measureBuiltinConstructors(names) {
  return names.map((name,index)=>{
    const C=globalThis[name];if(typeof C!=='function')return{name,present:false};
    class X extends C{};customElements.define('builtin-construction-'+index,X,{extends:'div'});
    function Fake() {}
    const calls=[()=>C(),()=>new C(),()=>new(class extends C{})(),()=>new X(),()=>Reflect.construct(C,[],X),()=>Reflect.construct(C,[],Fake),()=>Reflect.construct(HTMLElement,[],X)];
    const pd=Object.getOwnPropertyDescriptor(C,'prototype'),cd=Object.getOwnPropertyDescriptor(C.prototype,'constructor');
    const shape=[C.name,C.length,pd.writable,pd.enumerable,pd.configurable,cd.value===C,cd.writable,cd.enumerable,cd.configurable];
    return {name,present:true,shape,results:calls.map(fn=>{try{const e=fn();return[e.localName??null,e instanceof X,Object.getPrototypeOf(e)===Fake.prototype]}catch(e){return e.name}})};
  });
}
