/** Re-resolve selectors inside the owning page on every operation. */
export class Locator {
  constructor(context, selectors, index) {
    this._context = context;
    this._selectors = selectors;
    this._index = index;
  }
  locator(selector) {
    if (this._index !== undefined) throw new TypeError('Chain selectors before selecting nth()');
    return new Locator(this._context, [...this._selectors, selector]);
  }
  nth(index) {
    if (!Number.isInteger(index) || index < 0)
      throw new RangeError('index must be a nonnegative integer');
    return new Locator(this._context, this._selectors, index);
  }
  _run(op, options, command = {}) {
    const input = { op, selectors: this._selectors, ...command };
    if (this._index !== undefined) input.index = this._index;
    return this._context._automate(input, options);
  }
  waitFor({ state = 'visible', ...options } = {}) {
    return this._run('wait', options, { state });
  }
  count(options) {
    return this._run('count', options);
  }
  textContent(options) {
    return this._run('text', options);
  }
  getAttribute(name, options) {
    return this._run('attribute', options, { name });
  }
  click(options) {
    return this._run('input', options, { input: { action: 'click' } });
  }
  hover(options) {
    return this._run('input', options, { input: { action: 'hover' } });
  }
  fill(text, options) {
    return this._run('input', options, { input: { action: 'fill', text } });
  }
  typeText(text, options) {
    return this._run('input', options, { input: { action: 'type', text } });
  }
  press(key, { modifiers = 0, ...options } = {}) {
    return this._run('input', options, { input: { action: 'press', key, modifiers } });
  }
}

export class Frame {
  constructor(page, selectors) {
    this._page = page;
    this._selectors = selectors;
  }
  _automate(command, options) {
    return this._page._automate({ ...command, frames: this._selectors }, options);
  }
  frameLocator(selector) {
    return new Frame(this._page, [...this._selectors, selector]);
  }
  async frames() {
    return (await this._automate({ op: 'frames' })).map((selector) => this.frameLocator(selector));
  }
  url() {
    return this.evaluate('location.href');
  }
  locator(selector) {
    return new Locator(this, [selector]);
  }
  evaluate(source, options) {
    return this._automate({ op: 'evaluate', source }, options);
  }
  waitForFunction(source, { arg = null, ...options } = {}) {
    return this._automate({ op: 'function', source, arg }, options);
  }
  waitForSelector(selector, options) {
    return this.locator(selector).waitFor(options);
  }
  waitForLoadState(state = 'load', options) {
    if (!['load', 'domcontentloaded'].includes(state))
      throw new TypeError('state must be load or domcontentloaded');
    return this._automate({ op: 'load', state }, options);
  }
}
