import test from 'node:test';
import assert from 'node:assert/strict';
import { createSession } from '../../../scripts/coverage/session.mjs';

async function evaluate(source) {
  const session = await createSession();
  try {
    return await (await session.newPage()).evaluate(source);
  } finally {
    await session.close();
  }
}

test('selectors match elements, ancestors and static query results using canonical DOM state', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const root=document.createElement('section');root.id='selector-root';
    root.innerHTML='<div class="group"><span class="item" data-key="a"></span><span class="item"></span></div>';
    document.body.append(root);
    const leaf=root.querySelector('[data-key="a"]'),parent=leaf.parentElement,list=root.querySelectorAll('.item');
    const initial=[leaf.matches('.item[data-key="a"]'),leaf.webkitMatchesSelector('.item'),leaf.closest('.group')===parent,
      document.querySelector('#selector-root')===root,document.querySelectorAll('#selector-root .item').length,list instanceof NodeList];
    let conversions=0;const selector={toString(){conversions++;return 'section';}};
    leaf.matches=()=>{throw Error('public matches must not be called');};
    Object.defineProperty(leaf,'parentElement',{get(){throw Error('public parent must not be read');}});
    initial.push(leaf.closest(selector)===root,conversions);
    parent.append(document.createElement('span'));parent.firstChild.remove();
    initial.push(list.length,list[0]===leaf,root.querySelectorAll('.item').length);
    const shadow=root.attachShadow({mode:'open'}),child=document.createElement('b');shadow.append(child);
    initial.push(child.closest('section')===null,leaf.closest('.missing')===null);
    return initial;
  })()`),
    [true, true, true, true, 2, true, true, 1, 2, true, 1, true, true],
  );
});

test('selector methods require a selector and reject Symbols and invalid CSS', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const errors=[];
    for(const [receiver,names]of [[document,['querySelector','querySelectorAll']],
      [document.createElement('div'),['querySelector','querySelectorAll','matches','webkitMatchesSelector','closest']]]) {
      for(const name of names)for(const args of [[],[Symbol()],[']']]) {
        try{receiver[name](...args);errors.push('none');}catch(e){errors.push(e.name);}
      }
    }
    return errors;
  })()`),
    Array.from({ length: 7 }, () => ['TypeError', 'TypeError', 'SyntaxError']).flat(),
  );
});

test('part is a live token list on HTML and SVG elements through cloning and adoption', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const results=[];
    for(const e of [document.createElement('div'),document.createElementNS('http://www.w3.org/2000/svg','g')]) {
      const list=e.part;e.part='label label active';list.remove('active');list.add('icon');
      const clone=e.cloneNode();document.implementation.createHTMLDocument().adoptNode(e);
      results.push(list===e.part,list instanceof DOMTokenList,[...list],e.getAttribute('part'),clone.part.value,clone.part!==list);
      e.setAttribute('part','updated');results.push(list.value);
      try{list.supports('label');results.push('none');}catch(error){results.push(error.name);}
    }
    return results;
  })()`),
    Array.from({ length: 2 }, () => [
      true,
      true,
      ['label', 'icon'],
      'label icon',
      'label icon',
      true,
      'updated',
      'TypeError',
    ]).flat(),
  );
});

test('element fullscreen handlers register and cancel script-dispatched events', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const results=[];
    for(const e of [document.createElement('div'),document.createElementNS('http://www.w3.org/2000/svg','g')]) {
      for(const type of ['fullscreenchange','fullscreenerror','webkitfullscreenchange','webkitfullscreenerror']) {
        const key='on'+type;results.push(e[key]===null);let calls=0,receiver=false;
        e[key]=function(event){calls++;receiver=this===e&&event.target===e;return false;};
        const accepted=e.dispatchEvent(new Event(type,{cancelable:true}));
        e[key]=null;e.dispatchEvent(new Event(type));results.push(calls===1,receiver,!accepted);
      }
    }
    return results.every(Boolean);
  })()`),
    true,
  );
});

test('Safari-absent legacy fields remain absent on the corresponding DOM interfaces', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const failures=[];
    for(const [object,names] of [[document,['featurePolicy','onbeforexrselect','onfreeze','onpointerrawupdate','onresume','onsearch','wasDiscarded','webkitHidden','webkitVisibilityState']],
      [Element.prototype,['elementTiming','getInnerHTML','onsearch']],
      [HTMLElement.prototype,['autocapitalize','onbeforexrselect','onpointerrawupdate','virtualKeyboardPolicy']]]) {
      for(const name of names)if(name in object)failures.push(name);
    }
    return failures;
  })()`),
    [],
  );
});
