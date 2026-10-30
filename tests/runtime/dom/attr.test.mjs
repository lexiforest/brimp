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

test('Attr exposes names, namespaces, readonly metadata and ordinary interface identity', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const plain = document.createAttribute('TITLE'), namespaced = document.createAttributeNS('urn:test', 'p:label');
    const element = document.createElement('div'); element.setAttributeNodeNS(namespaced);
    const describe = attr => [attr.name, attr.localName, attr.prefix, attr.namespaceURI, attr.specified];
    const descriptors = ['name', 'localName', 'prefix', 'namespaceURI', 'specified', 'ownerElement', 'value'].map(name => {
      const d = Object.getOwnPropertyDescriptor(Attr.prototype, name);
      return [d.enumerable, d.configurable, typeof d.get, typeof d.set];
    });
    let error; try { new Attr(); } catch (e) { error = e.name; }
    return [describe(plain), describe(namespaced), plain.ownerElement, namespaced.ownerElement === element,
      plain.ownerDocument === document, Object.getPrototypeOf(plain) === Attr.prototype,
      Object.getPrototypeOf(Attr.prototype) === Node.prototype, Attr.prototype.constructor === Attr,
      Attr.prototype[Symbol.toStringTag], error, descriptors];
  })()`),
    [
      ['title', 'title', null, null, true],
      ['p:label', 'label', 'p', 'urn:test', true],
      null,
      true,
      true,
      true,
      true,
      true,
      'Attr',
      'TypeError',
      [
        ...Array.from({ length: 6 }, () => [true, true, 'function', 'undefined']),
        [true, true, 'function', 'function'],
      ],
    ],
  );
});

test('Attr value updates the owner attribute and emits mutation records', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const element = document.createElement('div'); element.setAttribute('title', 'old');
    element.setAttributeNS('urn:test', 'p:label', 'before');
    const attr = element.getAttributeNode('title'), ns = element.getAttributeNodeNS('urn:test', 'label');
    const observer = new MutationObserver(() => {});
    observer.observe(element, {attributes: true, attributeOldValue: true});
    attr.value = 'new'; ns.value = 'after';
    const records = observer.takeRecords().map(r => [r.attributeName, r.attributeNamespace, r.oldValue]);
    observer.disconnect();
    const values = [element.title, element.getAttributeNS('urn:test', 'label'), attr.value, ns.value];
    element.title = 'reflected'; const live = attr.value;
    const conversions = []; for (const v of [null, undefined, 42, '😀']) { attr.value = v; conversions.push(attr.value); }
    let error; try { attr.value = Symbol(); } catch (e) { error = e.name; }
    return [values, records, live, conversions, error, attr.value];
  })()`),
    [
      ['new', 'after', 'new', 'after'],
      [
        ['title', null, 'old'],
        ['label', 'urn:test', 'before'],
      ],
      'reflected',
      ['null', 'undefined', '42', '😀'],
      'TypeError',
      '😀',
    ],
  );
});

test('Attr removal retains the last value and does not reconnect old wrappers', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const element = document.createElement('div'); element.hidden = true;
    const removed = element.getAttributeNode('hidden');
    element.setAttribute('hidden', 'latest'); element.hidden = false;
    element.setAttribute('hidden', 'replacement');
    const replacement = element.getAttributeNode('hidden');
    const state = [removed.ownerElement, removed.value, replacement !== removed, replacement.ownerElement === element];
    removed.value = 'detached';
    const values = [removed.value, element.getAttribute('hidden')];
    const returned = element.removeAttributeNode(replacement);
    return [state, values, returned === replacement, replacement.ownerElement, replacement.value];
  })()`),
    [[null, 'latest', true, true], ['detached', 'replacement'], true, null, 'replacement'],
  );
});

test('Attr cloning, adoption and custom-element reactions retain useful DOM semantics', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const calls = [];
    customElements.define('attr-check', class extends HTMLElement {
      static get observedAttributes() { return ['data-value', 'flag']; }
      attributeChangedCallback(name, oldValue, value, namespace) { calls.push([name, oldValue, value, namespace]); }
    });
    const element = document.createElement('attr-check');
    const attr = document.createAttribute('data-value'); attr.value = 'one';
    element.setAttributeNode(attr); attr.value = 'two';
    const clone = attr.cloneNode(); clone.value = 'clone';
    const other = document.implementation.createHTMLDocument('other'); other.adoptNode(attr);
    element.setAttributeNS('urn:test', 'p:flag', 'yes');
    element.getAttributeNodeNS('urn:test', 'flag').value = 'no';
    element.removeAttributeNS('urn:test', 'flag');
    return [calls, clone.ownerElement, clone.value, clone.ownerDocument === document,
      attr.ownerElement, attr.value, attr.ownerDocument === other, element.hasAttribute('data-value')];
  })()`),
    [
      [
        ['data-value', null, 'one', null],
        ['data-value', 'one', 'two', null],
        ['data-value', 'two', null, null],
        ['flag', null, 'yes', 'urn:test'],
        ['flag', 'yes', 'no', 'urn:test'],
        ['flag', 'no', null, 'urn:test'],
      ],
      null,
      'clone',
      true,
      null,
      'two',
      true,
      false,
    ],
  );
});
