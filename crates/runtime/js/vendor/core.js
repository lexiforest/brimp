import * as streams from 'web-streams-polyfill';
for (const [name, value] of Object.entries(streams))
  Object.defineProperty(globalThis, name, { value, writable: true, configurable: true });
import structuredClone from 'core-js-pure/actual/structured-clone.js';
import xpath from 'xpath';
Object.defineProperty(globalThis, 'structuredClone', {
  value: structuredClone,
  writable: true,
  configurable: true,
});
class XPathEvaluator {
  createExpression(expression, resolver = null) {
    const compiled = xpath.createExpression(
      String(expression),
      typeof resolver === 'function' ? { lookupNamespaceURI: resolver } : resolver,
    );
    const evaluate = compiled.evaluate;
    compiled.evaluate = function (node, type = 0, result = null) {
      const doc = node.nodeType === 9 ? node : node.ownerDocument;
      this.context.allowAnyNamespaceForNoPrefix = doc?.contentType === 'text/html';
      return evaluate.call(this, node, type, result);
    };
    return compiled;
  }
  createNSResolver(node) {
    return xpath.createNSResolver(node);
  }
  evaluate(expression, node, resolver = null, type = 0, result = null) {
    return this.createExpression(expression, resolver).evaluate(node, type, result);
  }
}
Object.assign(globalThis, {
  XPathEvaluator,
  XPathResult: xpath.XPathResult,
  XPathExpression: xpath.XPathExpression,
});
for (const name of ['createExpression', 'createNSResolver', 'evaluate']) {
  Object.defineProperty(Document.prototype, name, {
    value: XPathEvaluator.prototype[name],
    writable: true,
    configurable: true,
  });
}
