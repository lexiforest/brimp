use brimp_runtime::runtime::native::{Session, SessionOptions};
use serde_json::json;

fn check(source: &str) {
    let session = Session::new(SessionOptions::default()).unwrap();
    let page = session.new_page().unwrap();
    assert_eq!(page.evaluate(source).unwrap(), json!(true));
}

#[test]
fn shared_microtask_order_and_observer_reentrancy() {
    check(
        r#"(async()=>{
      const order=[], e=document.createElement('div');
      const m=new MutationObserver(rs=>{order.push('observer:'+rs.length);if(!e.hasAttribute('y'))e.setAttribute('y','1')});
      m.observe(e,{attributes:true});
      queueMicrotask(()=>{order.push('queued');queueMicrotask(()=>order.push('nested'))});
      e.setAttribute('x','1');Promise.resolve().then(()=>order.push('promise'));
      await new Promise(r=>setTimeout(r,0));m.disconnect();
      if(order.join()!=='queued,observer:1,promise,nested,observer:1')throw Error(order);
      let type;try{queueMicrotask(null)}catch(e){type=e.name}return type==='TypeError';
    })()"#,
    );
}
#[test]
fn live_collections_and_namespace_attribute_identity() {
    check(
        r#"(()=>{
      const root=document.createElement('div'),nodes=root.childNodes,children=root.children;
      root.innerHTML='<b id="first"></b>';const old=root.firstChild;root.append(document.createTextNode('text'));
      if(nodes!==root.childNodes||nodes.length!==2||children!==root.children||children.first!==old||!Object.getOwnPropertyNames(children).includes('first'))throw Error('collection');
      old.remove();if(nodes.length!==1||children.length!==0||'first' in children)throw Error('stale collection');
      const a=document.createAttributeNS('urn:a','a:key');a.value='one';root.setAttributeNodeNS(a);
      const b=document.createAttributeNS('urn:b','b:key');b.value='two';root.setAttributeNodeNS(b);
      a.value='updated';if(root.getAttributeNS('urn:a','key')!=='updated'||root.attributes.getNamedItemNS('urn:b','key')!==b)throw Error('namespace isolation');
      const other=document.createElement('div');let name;try{other.setAttributeNode(a)}catch(e){name=e.name}if(name!=='InUseAttributeError')throw Error(name);
      root.removeAttributeNS('urn:a','key');root.setAttributeNS('urn:a','a:key','new');
      if(a.ownerElement!==null||a.value!=='updated'||root.getAttributeNodeNS('urn:a','key')===a)throw Error('detached attr');
      const foreign=document.implementation.createHTMLDocument();foreign.adoptNode(root);
      if(root.ownerDocument!==foreign||nodes[0].ownerDocument!==foreign||b.ownerDocument!==foreign)throw Error('adoption');
      const t=foreign.createElement('template');t.innerHTML='<template><b>x</b></template>';
      const content=t.content, originalOwner=content.ownerDocument;document.adoptNode(t);
      return content.ownerDocument!==originalOwner&&content.ownerDocument!==document&&content.firstChild.content.ownerDocument===content.ownerDocument&&document.forms===document.forms;
    })()"#,
    );
}
#[test]
fn replacement_validates_before_mutation_and_collapses_ranges() {
    check(
        r#"(()=>{
      const p=document.createElement('p');p.innerHTML='<i>abc</i><b>x</b>';
      const r=document.createRange();r.selectNodeContents(p.firstChild.firstChild);
      const replacement=document.createDocumentFragment();replacement.append(document.createElement('u'),document.createElement('em'));
      const observer=new MutationObserver(()=>{});observer.observe(p,{childList:true});const old=p.firstChild;
      p.replaceChild(replacement,old);
      const records=observer.takeRecords();
      if(r.startContainer!==p||r.startOffset!==0||!r.collapsed||replacement.childNodes.length)throw Error('range');
      if(records.length!==1||records[0].removedNodes[0]!==old||records[0].addedNodes.length!==2)throw Error('replacement records:'+records.length);
      const before=p.innerHTML;let error;try{p.replaceChild(document.createElement('q'),old)}catch(e){error=e.name}
      if(error!=='NotFoundError'||p.innerHTML!==before)throw Error('atomic failure');
      try{p.replaceChild(p,p.firstChild);return false}catch(e){if(e.name!=='HierarchyRequestError')throw e}
      return p.innerHTML===before;
    })()"#,
    );
}
#[test]
fn treewalker_skips_filters_without_crossing_root() {
    check(
        r#"(()=>{
      const root=document.createElement('div');root.innerHTML='<section><b></b><i></i></section><aside><q></q></aside><u></u>';
      const w=document.createTreeWalker(root,1,n=>n.localName==='section'?3:n.localName==='aside'?2:1);
      if(w.firstChild().localName!=='b'||w.nextSibling().localName!=='i'||w.nextSibling().localName!=='u'||w.previousSibling().localName!=='i')throw Error('siblings');
      if(w.parentNode()!==root||w.parentNode()!==null||w.lastChild().localName!=='u'||w.previousNode().localName!=='i'||w.previousNode().localName!=='b')throw Error('direction');
      let error;const recursive=document.createTreeWalker(root,1,()=>recursive.nextNode());try{recursive.nextNode()}catch(e){error=e.name}return error==='InvalidStateError';
    })()"#,
    );
}
#[test]
fn forms_validate_reset_and_preserve_selection() {
    check(
        r#"(()=>{
      document.body.innerHTML='<form id="f"><input name="a" required value="original"><fieldset disabled><legend><input name="legend" required></legend><input name="barred" required></fieldset></form><label for="n">Number</label><input id="n" form="f" type="number" min="2" step="2" value="4">';
      const f=document.getElementById('f'),a=f.elements.namedItem('a'),n=document.getElementById('n'),validity=n.validity;
      if(n.form!==f||n.labels[0].control!==n||f.elements.length!==5)throw Error('association');
      n.value='5';if(n.validity!==validity||!validity.stepMismatch||n.checkValidity())throw Error('step validity');n.stepUp();if(n.value!=='6')throw Error('step');
      n.setCustomValidity('bad');if(n.validationMessage!=='bad'||!n.validity.customError)throw Error('custom');n.setCustomValidity('');
      a.value='edited';const cancel=e=>e.preventDefault();f.addEventListener('reset',cancel);f.reset();if(a.value!=='edited')throw Error('canceled reset');f.removeEventListener('reset',cancel);f.reset();
      a.defaultValue='changed';if(a.value!=='changed'||n.value!=='4')throw Error('dirty flags');
      a.value='abcdef';a.setSelectionRange(4,6);a.setRangeText('X',1,3);if(a.value!=='aXdef'||a.selectionStart!==3||a.selectionEnd!==5)throw Error('selection');
      let error;try{n.setRangeText('x')}catch(e){error=e.name}
      return error==='InvalidStateError'&&!f.elements.namedItem('barred').willValidate&&f.elements.namedItem('legend').willValidate;
    })()"#,
    );
}
#[test]
fn safari_element_construction_and_internals_lifecycle() {
    check(
        r#"(()=>{
      class Button extends HTMLButtonElement{constructor(){super();this.marker=true}}
      customElements.define('test-button',Button,{extends:'button'});
      try { new Button(); return false; } catch(error) { if(error.name !== 'TypeError') return false; }
      const created=document.createElement('button',{is:'test-button'});
      if(created instanceof Button || created.hasAttribute('is') || created.marker)throw Error('unsupported customized built-in');
      const root=document.createElement('div');root.innerHTML='<button is="test-button"></button>';customElements.upgrade(root);
      if(root.firstChild instanceof Button)throw Error('unsupported upgrade');
      class Autonomous extends HTMLElement{constructor(){super();this.internals=this.attachInternals()}}
      customElements.define('test-autonomous',Autonomous);const e=new Autonomous();e.internals.role='button';e.internals.states.add('expanded');
      const shadow=e.attachShadow({mode:'closed'});if(e.internals.shadowRoot!==shadow||e.hasAttribute('role')||!e.internals.states.has('expanded'))throw Error('internals');
      try{e.attachInternals();return false}catch(err){return err.name==='NotSupportedError'}
    })()"#,
    );
}
#[test]
fn stylesheet_lists_rules_and_adoption_preserve_ownership() {
    check(
        r#"(async()=>{
      const style=document.createElement('style');style.textContent='p {color:red !important}';document.head.append(style);
      const sheet=style.sheet,list=sheet.cssRules,rule=list[0];
      if(sheet.ownerNode!==style||rule.parentStyleSheet!==sheet||rule.style.parentRule!==rule)throw Error('ownership');
      rule.style.setProperty('color','blue','important');if(rule.style.color!=='blue'||rule.style.getPropertyPriority('color')!=='important')throw Error('priority');
      sheet.insertRule('@media screen { b { color:red } }',1);const group=list[1];group.insertRule('i {color:blue}',1);
      if(list!==sheet.cssRules||group.cssRules[1].parentRule!==group||group.cssRules[1].parentStyleSheet!==sheet)throw Error('group ownership');
      const removed=group.cssRules[0];group.deleteRule(0);if(removed.parentRule!==null||removed.parentStyleSheet!==null)throw Error('detached rule');
      let error;try{sheet.insertRule('b{}',10)}catch(e){error=e.name}if(error!=='IndexSizeError')throw Error(error);
      const constructed=new CSSStyleSheet();await constructed.replace('b {color:red}');const retained=constructed.cssRules;
      constructed.replaceSync('i {color:blue}');if(retained!==constructed.cssRules||retained[0].selectorText!=='i')throw Error('replace list');
      document.adoptedStyleSheets.push(constructed);const shadow=document.body.attachShadow({mode:'open'});shadow.adoptedStyleSheets=[constructed];
      try{document.adoptedStyleSheets=[sheet];return false}catch(e){if(e.name!=='NotAllowedError')throw e}
      style.textContent='u {color:green}';return style.sheet!==sheet&&style.sheet.cssRules[0].selectorText==='u'&&shadow.adoptedStyleSheets[0]===constructed;
    })()"#,
    );
}
#[test]
fn document_writing_is_incremental_and_reentrant() {
    check(
        r#"(()=>{
      const old=document.body;document.open();document.write('<!doctype html><title>new</title><div id="root"><b id="ke');
      document.write('pt">hello</b>');const b=document.getElementById('kept');
      document.write('<script>document.write("<i>nested</i>");globalThis.trailingHidden=!document.querySelector("u")</script><u>after</u></div>');document.close();
      if(old.isConnected||document.title!=='new'||document.getElementById('kept')!==b||b.textContent!=='hello')throw Error('identity');
      const root=document.getElementById('root');if(!trailingHidden||root.querySelector('i').nextSibling.localName!=='u')throw Error('script order');
      document.open();document.writeln('<p>again</p>');document.close();return document.querySelector('p').textContent==='again'&&document.readyState==='complete';
    })()"#,
    );
}

#[test]
fn linked_stylesheets_load_and_guard_cross_origin_rules() {
    use async_trait::async_trait;
    use brimp_runtime::{
        network::{
            NetworkError, ResourceLoader, ResourceRequest, ResourceResponse, ResponseMetadata,
        },
        runtime::native::RequestOptions,
    };
    use http::{HeaderMap, HeaderValue, StatusCode};
    use std::sync::Arc;
    struct Styles;
    #[async_trait]
    impl ResourceLoader for Styles {
        async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
            let css = request.url.ends_with(".css");
            let mut headers = HeaderMap::new();
            headers.insert(
                "content-type",
                HeaderValue::from_static(if css { "text/css" } else { "text/html" }),
            );
            Ok(ResourceResponse {
                status: StatusCode::OK,
                headers: headers.into(),
                body: if css {
                    b"b {color: red}".to_vec()
                } else {
                    b"<!doctype html><html><head></head><body></body></html>".to_vec()
                },
                effective_url: request.url,
                metadata: ResponseMetadata::default(),
            })
        }
    }
    let session =
        Session::with_resource_loader(SessionOptions::default(), Arc::new(Styles)).unwrap();
    let page = session.new_page().unwrap();
    page.request("GET", "https://sheet.test/", RequestOptions::default())
        .unwrap();
    assert_eq!(page.evaluate(r#"(async()=>{
      const load=async href=>{const link=document.createElement('link');link.rel='stylesheet';link.href=href;const loaded=new Promise((r,j)=>{link.onload=r;link.onerror=j});document.head.append(link);await loaded;return link};
      const link=await load('/same.css'),sheet=link.sheet;
      if(sheet.ownerNode!==link||sheet.cssRules[0].style.color!=='red'||document.styleSheets[0]!==sheet)throw Error('load');
      link.remove();if(link.sheet!==null||document.styleSheets.length)throw Error('detach');
      const cross=await load('https://other.test/cross.css');let error;try{cross.sheet.cssRules}catch(e){error=e.name}
      return error==='SecurityError'&&document.styleSheets.length===1;
    })()"#).unwrap(),json!(true));
}
