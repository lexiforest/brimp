const __writingDocuments = new WeakSet(),
  __documentWriterEval = globalThis.eval;
function __deliverWriteResult(doc, serialized) {
  while (serialized !== null) {
    const result = JSON.parse(serialized);
    __deliverParserMutations(JSON.stringify(result.records));
    if (result.script === null) break;
    const script = __callHost('mutationNode', window, String(result.script));
    __startedScripts.add(script);
    if (
      doc === document &&
      (!script.type || ['text/javascript', 'application/javascript'].includes(script.type))
    ) {
      if (script.hasAttribute('src'))
        throw new DOMException(
          'External scripts in document.write are not supported',
          'NotSupportedError',
        );
      const previous = __currentScript;
      __currentScript = script;
      try {
        __documentWriterEval(script.textContent);
      } catch (error) {
        setTimeout(() => {
          throw error;
        }, 0);
      } finally {
        __currentScript = previous;
      }
    }
    serialized = __callHost('documentWriteNext', doc);
  }
}
Document.prototype.open = function () {
  const error = __callHost('documentWriteError', this);
  if (error) throw new DOMException('The document cannot be opened for writing', error);
  if (arguments.length >= 3)
    throw new DOMException('Window-opening overload is unavailable', 'NotSupportedError');
  if (this === document && __currentScript) return this;
  for (const node of [this, ...this.querySelectorAll('*')]) __eventListeners.delete(node);
  while (this.firstChild) this.removeChild(this.firstChild);
  __callHost('documentOpen', this, this !== document && __documentWindow(this) !== null);
  __writingDocuments.add(this);
  if (this === document) __documentReadyState = 'loading';
  return this;
};
Document.prototype.write = function (...text) {
  if (!__writingDocuments.has(this)) {
    if (this === document && __currentScript)
      throw new DOMException(
        'Writing into a navigation parser is unavailable',
        'NotSupportedError',
      );
    this.open();
  }
  __deliverWriteResult(this, __callHost('documentWrite', this, text.map(String).join('')));
};
Document.prototype.writeln = function (...text) {
  this.write(...text, '\n');
};
Document.prototype.close = function () {
  if (!__writingDocuments.has(this)) return;
  __deliverWriteResult(this, __callHost('documentClose', this));
  __writingDocuments.delete(this);
  if (this === document) {
    __documentReadyState = 'interactive';
    this.dispatchEvent(new Event('readystatechange'));
    this.dispatchEvent(new Event('DOMContentLoaded', { bubbles: true }));
    __documentReadyState = 'complete';
    this.dispatchEvent(new Event('readystatechange'));
    window.dispatchEvent(new Event('load'));
  }
};
