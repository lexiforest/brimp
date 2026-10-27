import test from 'node:test';
import assert from 'node:assert/strict';
import {createSession} from '../../../bindings/node/index.js';

async function evaluate(source) {
  const session = await createSession();
  try { return await (await session.newPage()).evaluate(source); }
  finally { await session.close(); }
}

test('Document creation, adoption, collections and traversal share canonical DOM nodes', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const parsed=new DOMParser().parseFromString('<!doctype html><p id="p" class="item" name="item">text</p>','text/html');
    const p=parsed.getElementById('p'),copy=document.importNode(p,true);
    document.body.append(copy);const live=document.getElementsByClassName('item');
    const moved=document.adoptNode(p);document.body.append(moved);
    const fragment=document.createDocumentFragment();fragment.append(document.createComment('comment'),document.createTextNode('tail'));
    moved.append(fragment);const attr=document.createAttribute('title');attr.value='value';moved.setAttributeNode(attr);
    const nsattr=document.createAttributeNS('urn:test','x:value');nsattr.value='namespaced';moved.setAttributeNodeNS(nsattr);
    const svg=document.createElementNS('http://www.w3.org/2000/svg','svg');document.body.append(svg);
    const walker=document.createTreeWalker(moved,NodeFilter.SHOW_TEXT),iterator=document.createNodeIterator(moved,NodeFilter.SHOW_ELEMENT);
    return [moved===p,p.ownerDocument===document,parsed.getElementById('p'),copy!==p,copy.textContent,live.length,
      document.getElementsByName('item').length,document.getElementsByTagName('p').length,
      document.getElementsByTagNameNS('http://www.w3.org/2000/svg','svg')[0]===svg,
      walker.nextNode().data,iterator.nextNode()===p,attr.ownerElement===p,p.getAttributeNS('urn:test','value'),p.lastChild.data];
  })()`),[true,true,null,true,'text',2,2,2,true,'text',true,true,'namespaced','tail']);
});

test('Document editing state and root/focus relationships stay local to each document', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const other=document.implementation.createHTMLDocument('other'),empty=new Document();
    const button=document.createElement('button');document.body.append(button);button.focus();
    const focused=[document.activeElement===button,other.activeElement===other.body];button.remove();
    const edit=[document.designMode,other.designMode];document.designMode='ON';edit.push(button.isContentEditable,other.designMode);
    document.designMode='invalid';edit.push(document.designMode);document.designMode='off';edit.push(button.isContentEditable);
    const svg=empty.createElementNS('http://www.w3.org/2000/svg','svg');empty.append(svg);
    const roots=[empty.rootElement===svg,document.rootElement,other.scrollingElement===other.documentElement];
    const quirks=new DOMParser().parseFromString('<body>quirks</body>','text/html');roots.push(quirks.scrollingElement===quirks.body);
    empty.replaceChildren(empty.createComment('before'),svg);empty.prepend(empty.createComment('first'));
    return [focused,document.activeElement===document.body,edit,roots,empty.childNodes.length,
      document.defaultView===window,other.defaultView,other.currentScript,other.readyState,other.title];
  })()`),[[true,true],true,['off','off',true,'off','on',false],[true,null,true,true],3,true,null,null,'complete','other']);
});

test('Document writing, XPath, ranges, events and stylesheet state remain functional', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const doc=document.implementation.createHTMLDocument();const opened=doc.open()===doc;
    doc.write('<!doctype html><html><head></head><body><p id="p">hello</p>');doc.writeln('<p>world</p>');doc.close();
    const p=doc.getElementById('p'),range=doc.createRange();range.selectNodeContents(p);
    const selection=document.getSelection();selection.removeAllRanges();
    document.body.append(document.importNode(p,true));const selected=document.createRange();selected.selectNodeContents(document.body.firstChild);selection.addRange(selected);
    const expression=doc.createExpression('count(//p)'),resolver=doc.createNSResolver(doc.documentElement);
    const event=doc.createEvent('Event');event.initEvent('family',false,false);let calls=0;doc.addEventListener('family',()=>calls++);doc.dispatchEvent(event);
    const style=document.createElement('style');style.textContent='p { color: red; }';document.head.append(style);
    const sheet=new CSSStyleSheet();sheet.replaceSync('p { color: blue; }');document.adoptedStyleSheets=[sheet];
    doc.bgColor='red';doc.fgColor='blue';doc.linkColor='green';doc.alinkColor='yellow';doc.vlinkColor='purple';
    return [opened,doc.getElementsByTagName('p').length,range.toString(),selection.toString(),
      expression.evaluate(doc,XPathResult.NUMBER_TYPE,null).numberValue,typeof resolver.lookupNamespaceURI,
      doc.evaluate('string(//p[@id="p"])',doc,null,XPathResult.STRING_TYPE,null).stringValue,calls,
      document.styleSheets[0]===style.sheet,document.adoptedStyleSheets[0]===sheet,
      [doc.bgColor,doc.fgColor,doc.linkColor,doc.alinkColor,doc.vlinkColor]];
  })()`),[true,2,'hello','hello',2,'function','hello',1,true,true,['red','blue','green','yellow','purple']]);
});

test('Document heavy services preserve explicit placeholders without producing browser effects', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const d=document;let cdata;try{d.createCDATASection('text')}catch(e){cdata=e.name}
    const before=d.body.innerHTML;const pi=d.createProcessingInstruction('target','data');
    const noops=[d.exitFullscreen(),d.exitPictureInPicture(),d.exitPointerLock(),d.webkitExitFullscreen(),d.webkitCancelFullScreen()];
    let illegal;try{new CDATASection()}catch(e){illegal=e.name}
    return [['all','fragmentDirective','timeline','lastModified'].map(n=>n in d&&d[n]===undefined),
      [d.fullscreen,d.webkitIsFullScreen,d.fullscreenEnabled,d.webkitFullscreenEnabled,d.pictureInPictureEnabled],
      [d.fullscreenElement,d.webkitFullscreenElement,d.webkitCurrentFullScreenElement,d.pictureInPictureElement,d.pointerLockElement],
      [d.referrer,d.xmlEncoding,d.xmlStandalone,d.xmlVersion,d.hidden,d.visibilityState,d.hasFocus()],
      cdata,pi===undefined,noops.every(v=>v===undefined),before===d.body.innerHTML,
      illegal,CDATASection.prototype instanceof Text,HTMLDocument===Document,d instanceof HTMLDocument,
      Object.prototype.toString.call(new DOMParser())];
  })()`),[[true,true,true,true],[false,false,true,true,true],[null,null,null,null,null],
    ['',null,false,null,false,'visible',true],'NotSupportedError',true,true,true,'TypeError',true,true,true,'[object DOMParser]']);
});
