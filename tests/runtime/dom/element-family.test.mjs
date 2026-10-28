import test from 'node:test';
import assert from 'node:assert/strict';
import {createSession} from '../../../bindings/node/index.js';

async function evaluate(source, plugins = []) {
  const session = await createSession({plugins});
  try { return await (await session.newPage()).evaluate(source); }
  finally { await session.close(); }
}

test('Element attribute, fragment and collection workflows retain canonical DOM state', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const el=document.createElement('section');document.body.append(el);
    el.id='root';el.className='first';el.classList.add('second');el.dataset.task='value';
    el.setAttribute('data-a','a');el.setAttributeNS('urn:test','x:value','namespaced');
    const attribute=document.createAttribute('title');attribute.value='local';el.setAttributeNode(attribute);
    const attrs=[el.hasAttribute('data-a'),el.getAttribute('title'),el.getAttributeNode('title')===attribute,
      el.getAttributeNS('urn:test','value'),el.hasAttributeNS('urn:test','value'),el.getAttributeNodeNS('urn:test','value').value,
      el.hasAttributes(),el.getAttributeNames().includes('data-a'),el.attributes.getNamedItem('title')===attribute];
    el.innerHTML='<b class="item">one</b>';const live=el.getElementsByClassName('item');
    el.insertAdjacentHTML('beforeend','<i class="item">two</i>');el.insertAdjacentText('afterbegin','start');
    const em=document.createElement('em');el.insertAdjacentElement('beforeend',em);em.outerHTML='<u>three</u>';
    const children=[live.length,el.getElementsByTagName('b').length,el.getElementsByTagNameNS('http://www.w3.org/1999/xhtml','i').length,
      el.localName,el.namespaceURI,el.prefix,el.tagName,el.id,el.className,el.dataset.task];
    el.removeAttributeNode(attribute);el.removeAttributeNS('urn:test','value');el.removeAttribute('data-a');
    const toggle=[el.toggleAttribute('hidden'),el.toggleAttribute('hidden')];
    el.replaceChildren('middle');el.prepend('before');el.append('after');
    return [attrs,children,toggle,el.textContent,attribute.ownerElement,live.length];
  })()`),[[true,'local',true,'namespaced',true,'namespaced',true,true,true],
    [2,1,1,'section','http://www.w3.org/1999/xhtml',null,'SECTION','root','first second','value'],[true,false],'beforemiddleafter',null,0]);
});

test('HTML editability, nonce, text replacement and focus/click state remain local DOM behavior', async () => {
  assert.deepEqual(await evaluate(String.raw`(() => {
    const parent=document.createElement('div'),el=document.createElement('span');parent.append(el);document.body.append(parent);
    const edit=[el.isContentEditable];parent.contentEditable='true';edit.push(el.isContentEditable);
    el.contentEditable='false';edit.push(el.isContentEditable);el.contentEditable='inherit';edit.push(el.isContentEditable);
    parent.contentEditable='plaintext-only';edit.push(el.isContentEditable);
    el.nonce='token';const clone=el.cloneNode();el.innerText='before';const read=el.outerText;el.outerText='first\r\nsecond';
    const button=document.createElement('button');parent.append(button);let clicks=0;button.onclick=()=>clicks++;
    button.focus();const focused=document.activeElement===button;button.click();button.blur();
    return [edit,clone.nonce,clone.getAttribute('nonce'),read,parent.innerHTML.startsWith('first<br>second'),el.parentNode,
      focused,clicks,document.activeElement!==button];
  })()`),[[false,true,false,true,true],'token','token','before',true,null,true,1,true]);
});

test('SVG namespace relationships and reflection work while geometry remains explicit placeholders', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const ns='http://www.w3.org/2000/svg',svg=document.createElementNS(ns,'svg'),group=document.createElementNS(ns,'g'),
      rect=document.createElementNS(ns,'rect'),title=document.createElementNS(ns,'title');
    svg.append(group);group.append(rect,title);document.body.append(svg);
    const cls=rect.className;cls.baseVal='shape';rect.classList.add('selected');
    rect.autofocus=true;rect.tabIndex=3;rect.nonce='svg-token';rect.dataset.item='rect';rect.style.color='red';
    const clone=rect.cloneNode();
    return [svg instanceof SVGGraphicsElement,rect instanceof SVGGraphicsElement,rect instanceof SVGElement,
      title instanceof SVGElement,!(title instanceof SVGGraphicsElement),rect.ownerSVGElement===svg,rect.viewportElement===svg,
      svg.ownerSVGElement,cls===rect.className,cls.baseVal,cls.animVal,Object.prototype.toString.call(cls),
      clone.autofocus,clone.tabIndex,clone.nonce,clone.dataset.item,clone.style.color,
      rect.requiredExtensions,rect.systemLanguage,rect.transform,
      Object.prototype.toString.call(rect.getBBox()),[rect.getBBox().x,rect.getBBox().width],
      [rect.getCTM().a,rect.getCTM().b,rect.getCTM().c,rect.getCTM().d,rect.getCTM().e,rect.getCTM().f],
      Object.prototype.toString.call(rect.getScreenCTM()),
      !('nearestViewportElement' in rect),!('farthestViewportElement' in rect)];
  })()`),[true,true,true,true,true,true,true,null,true,'shape selected','shape selected','[object SVGAnimatedString]',
    true,3,'svg-token','rect','red','','','','[object DOMRect]',[0,0],[1,0,0,1,0,0],'[object DOMMatrix]',true,true]);
});

test('SVG element classes follow the shape hierarchy and expose non-rendering geometry', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const ns='http://www.w3.org/2000/svg',svg=document.createElementNS(ns,'svg'),rect=document.createElementNS(ns,'rect'),
      group=document.createElementNS(ns,'g'),path=document.createElementNS(ns,'path'),text=document.createElementNS(ns,'text');
    svg.append(group,rect,path,text);
    const point=svg.createSVGPoint();
    point.x=5;point.y=6;
    const transformed=point.matrixTransform(svg.createSVGMatrix().translate(10,20));
    return [svg instanceof SVGSVGElement,rect instanceof SVGGeometryElement,path instanceof SVGGeometryElement,
      group instanceof SVGGeometryElement,text instanceof SVGGeometryElement,
      svg instanceof SVGGeometryElement,rect instanceof SVGGraphicsElement,
      [point.x,point.y,point.z,point.w],Object.prototype.toString.call(point),
      [transformed.x,transformed.y],Object.prototype.toString.call(svg.createSVGMatrix()),
      [rect.getBBox().x,rect.getBBox().y,rect.getBBox().width,rect.getBBox().height],
      rect.isPointInFill(point),rect.isPointInStroke(point),rect.getTotalLength(),
      Object.prototype.toString.call(path.getPointAtLength(3)),Object.prototype.toString.call(svg.createSVGRect())];
  })()`),[true,true,true,false,false,false,true,[5,6,0,1],'[object SVGPoint]',[15,26],'[object DOMMatrix]',
    [0,0,0,0],false,false,0,'[object SVGPoint]','[object DOMRect]']);
});

test('Element geometry and computed-style placeholders do not alter styles or emit scroll events', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const el=document.createElement('div');el.style.width='100px';document.body.append(el);let events=0;el.onscroll=()=>events++;
    el.scrollTop=12;el.scrollLeft=7;
    const map=el.attributeStyleMap;map.set('height',CSS.px(5));
    return [['clientHeight','clientWidth','clientTop','clientLeft','scrollHeight','scrollWidth'].map(n=>el[n]),
      el.scrollTop,el.scrollLeft,events,el.computedStyleMap()===undefined,el.style.width,map===el.attributeStyleMap,el.style.height];
  })()`,[]),[[0,0,0,0,0,0],12,7,0,true,'100px',true,'5px']);
});
