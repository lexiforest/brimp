import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createSession} from '../../../scripts/coverage/session.mjs';
import {inputStringFields, measureInputStringCodeUnits} from '../../../scripts/coverage/input-string-cases.mjs';

async function evaluate(source) {
  const session = await createSession();
  try { return await (await session.newPage()).evaluate(source); }
  finally { await session.close(); }
}

test('input string reflections preserve the measured Safari surrogate cases', async () => {
  const reference = JSON.parse(fs.readFileSync(new URL('../fixtures/input-string.safari.json', import.meta.url)));
  assert.deepEqual(await evaluate(`(${measureInputStringCodeUnits})(${JSON.stringify(inputStringFields)})`), reference.cases);
});

test('native attributes round-trip every UTF-16 code unit and long values', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const failures = [], input = document.createElement('input');
    let value = '';
    for (let unit = 0; unit <= 0xffff; ++unit) value += String.fromCharCode(unit) + 'x';
    input.name = value;
    if (input.name !== value || input.getAttribute('name') !== value || input.attributes.name.value !== value)
      failures.push('code-unit round trip');
    const copy = input.cloneNode();
    if (copy.name !== value || copy.getAttribute('name') !== value) failures.push('native clone');
    input.name = ''; if (input.name !== '' || input.getAttribute('name') !== '') failures.push('empty');
    if (copy.name !== value) failures.push('clone independence');
    return failures;
  })()`), []);
});

test('input reflections preserve code units through every attribute setter, cloning and import', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const failures = [], foreign = document.implementation.createHTMLDocument();
    const samples = [String.fromCharCode(0xd800), String.fromCharCode(0xdc00),
      String.fromCharCode(0xd800, 0xd800, 0xdc00, 0xdc00),
      String.fromCharCode(0, 0xdc00, 0xd800, 0x26, 0x22), '你好🦀'];
    for (const name of ${JSON.stringify(inputStringFields)}) for (const sample of samples) {
      const value = name === 'autocomplete' ? 'section-' + sample + ' email' : sample;
      const attribute = name.toLowerCase(), input = document.createElement('input');
      const setters = [
        () => { input[name] = value; },
        () => input.setAttribute(attribute, value),
        () => input.setAttributeNS(null, attribute, value),
        () => { input.getAttributeNode(attribute).value = value; },
        () => { input.getAttributeNode(attribute).nodeValue = value; },
        () => { input.getAttributeNode(attribute).textContent = value; },
        () => { const attr = document.createAttribute(attribute); attr.value = value; input.setAttributeNode(attr); },
      ];
      for (let method = 0; method < setters.length; ++method) {
        input[name] = 'before'; setters[method]();
        for (const target of [input, input.cloneNode(true), foreign.importNode(input, true)]) {
          if (target[name] !== value || target.getAttribute(attribute) !== value ||
              target.getAttributeNS(null, attribute) !== value || target.attributes.getNamedItem(attribute).value !== value)
            failures.push([name, method, 'attribute or clone']);
        }
        const attr = input.getAttributeNode(attribute);
        input.removeAttributeNode(attr);
        if (attr.value !== value || attr.ownerElement !== null) failures.push([name, method, 'detached Attr']);
        input.setAttributeNode(attr);
      }
    }
    return failures;
  })()`), []);
});

test('namespaced attributes, mutation records and HTML serialization retain surrogate code units', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const failures = [], input = document.createElement('input');
    const value = String.fromCharCode(0xd800, 0, 0xd83e, 0xdd80, 0xdc00), next = value + 'next';
    const observer = new MutationObserver(() => {});
    observer.observe(input, {attributes: true, attributeOldValue: true});
    input.name = value; input.name = value; input.name = next;
    input.setAttributeNS('urn:test', 't:name', value);
    input.getAttributeNodeNS('urn:test', 'name').value = next;
    const expected = [['name', null, null], ['name', value, null], ['name', value, null],
      ['name', null, 'urn:test'], ['name', value, 'urn:test']];
    const actual = observer.takeRecords().map(r => [r.attributeName, r.oldValue, r.attributeNamespace]);
    if (JSON.stringify(actual) !== JSON.stringify(expected)) failures.push('mutation oldValue');
    observer.disconnect();
    const clone = input.cloneNode();
    if (clone.getAttributeNS('urn:test', 'name') !== next || clone.name !== next) failures.push('namespace clone');
    input.removeAttributeNS('urn:test', 'name');
    input.name = value + '&"';
    if (input.outerHTML !== '<input name="' + value + '&amp;&quot;">') failures.push('outerHTML');
    const parent = document.createElement('div'); parent.append(input);
    if (parent.innerHTML !== input.outerHTML) failures.push('innerHTML');
    const other = document.implementation.createHTMLDocument(); other.adoptNode(input);
    if (input.name !== value + '&"' || input.ownerDocument !== other) failures.push('adoption');
    return failures;
  })()`), []);
});

test('ID, name and class lookups distinguish surrogates from replacement characters', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const failures = [], values = [String.fromCharCode(0xd800), String.fromCharCode(0xdc00), '�', '🦀'];
    const nodes = values.map(value => {
      const input = document.createElement('input'); input.id = value; input.name = value;
      input.className = value + ' shared'; document.body.append(input); return input;
    });
    for (let index = 0; index < values.length; ++index) {
      const value = values[index], node = nodes[index];
      if (document.getElementById(value) !== node || document.getElementsByName(value)[0] !== node ||
          document.getElementsByClassName(value + ' shared')[0] !== node) failures.push(index);
    }
    // Safari normalizes surrogate code units in CSS selectors to U+FFFD,
    // whereas DOMString name/ID/class lookups preserve them.
    if (document.querySelector('[name="' + values[0] + '"]') !== nodes[2]) failures.push('CSS preprocessing');
    return failures;
  })()`), []);
});

test('custom element reactions receive unchanged UTF-16 attribute values synchronously', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const log = [], value = String.fromCharCode(0xd800, 0, 0xdc00);
    class AttributeStrings extends HTMLElement {
      static get observedAttributes() { return ['data-value']; }
      attributeChangedCallback(name, oldValue, newValue) { log.push([name, oldValue, newValue]); }
    }
    customElements.define('attribute-strings', AttributeStrings);
    const element = document.createElement('attribute-strings');
    element.setAttribute('data-value', value);
    element.getAttributeNode('data-value').value = value;
    element.removeAttribute('data-value');
    const expected = [['data-value', null, value], ['data-value', value, value], ['data-value', value, null]];
    return JSON.stringify(log) === JSON.stringify(expected) ? [] : ['reaction values'];
  })()`), []);
});

test('URL getters use scalar conversion while content attributes retain original code units', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const failures = [];
    for (const unit of [0xd800, 0xdc00]) {
      const value = 'https://example.test/' + String.fromCharCode(unit);
      const anchor = document.createElement('a'); anchor.setAttribute('href', value);
      if (anchor.getAttribute('href') !== value) failures.push('raw attribute');
      if (anchor.href !== 'https://example.test/%EF%BF%BD') failures.push(['URL', anchor.href]);
    }
    return failures;
  })()`), []);
});
