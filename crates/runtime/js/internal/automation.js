// Host-only polling function. Its state is rooted by Rust, never installed on window.
function pollAutomation(input, state, prepareInput) {
  const command = JSON.parse(input);
  const pending = () => ({ done: false });
  let realm = window;
  for (const selector of command.frames || []) {
    const matches = realm.document.querySelectorAll(selector);
    if (matches.length > 1)
      return { error: `Frame selector matched ${matches.length} elements: ${selector}` };
    if (!matches.length) return pending();
    realm = matches[0].contentWindow;
    if (!realm) return { error: `Not a frame: ${selector}` };
  }
  if (command.op === 'frames') {
    const paths = Array.from(realm.document.querySelectorAll('iframe,frame'), (frame) => {
      const parts = [];
      for (let element = frame; element; element = element.parentElement) {
        const siblings = element.parentElement?.children || element.ownerDocument.children;
        parts.unshift(
          `${element.localName}:nth-child(${Array.from(siblings).indexOf(element) + 1})`,
        );
      }
      return parts.join(' > ');
    });
    return { done: true, value: paths };
  }
  if (command.op === 'evaluate' || command.op === 'function') {
    if (!state.started) {
      state.started = true;
      let value = realm.eval(command.source);
      if (command.op === 'function' && typeof value === 'function') value = value(command.arg);
      state.result = { done: false };
      const resolve = (value) => {
        state.value = value;
        state.ready = true;
        state.result.value = value;
        state.result.done = command.op === 'evaluate' || !!value;
      };
      if (value != null && typeof value.then === 'function') {
        Promise.resolve(value).then(resolve, (error) => {
          state.failed = true;
          state.error = String(error);
          state.ready = true;
        });
      } else resolve(value);
    }
    if (!state.ready) return state.result;
    if (state.failed) throw new Error(state.error);
    if (command.op === 'evaluate' || state.value) return { done: true, value: state.value };
    state.started = state.ready = false;
    return pending();
  }
  if (command.op === 'load') {
    const current = realm.document.readyState;
    return {
      done: command.state === 'domcontentloaded' ? current !== 'loading' : current === 'complete',
      value: null,
    };
  }
  let matches = [realm.document];
  for (const selector of command.selectors || [])
    matches = matches.flatMap((root) => Array.from(root.querySelectorAll(selector)));
  if (command.index !== undefined) matches = matches.slice(command.index, command.index + 1);
  if (command.op === 'count') return { done: true, value: matches.length };
  if (matches.length > 1) return { error: `Strict locator matched ${matches.length} elements` };
  const target = matches[0];
  const visible = (element) => {
    if (!element || !element.isConnected) return false;
    const style = realm.getComputedStyle(element);
    const rect = element.getBoundingClientRect();
    return (
      style.display !== 'none' &&
      style.visibility !== 'hidden' &&
      style.visibility !== 'collapse' &&
      rect.width > 0 &&
      rect.height > 0
    );
  };
  if (command.op === 'wait') {
    const states = {
      attached: !!target,
      detached: !target,
      visible: visible(target),
      hidden: !visible(target),
    };
    if (!(command.state in states)) return { error: `Unknown selector state: ${command.state}` };
    return { done: states[command.state], value: null };
  }
  if (!target) return pending();
  if (command.op === 'text') return { done: true, value: target.textContent };
  if (command.op === 'attribute') return { done: true, value: target.getAttribute(command.name) };
  if (command.op !== 'input') return { error: `Unknown automation operation: ${command.op}` };
  if (!visible(target) || target.matches(':disabled') || target.closest('[inert]'))
    return pending();
  if (['fill', 'type'].includes(command.input.action)) {
    if (target.readOnly) return pending();
    if (!target.matches('input,textarea') && !target.isContentEditable)
      return { error: 'Locator target is not editable' };
  }
  prepareInput?.(target);
  const rect = target.getBoundingClientRect();
  const box = JSON.stringify([rect.x, rect.y, rect.width, rect.height]);
  const stable = state.target === target && state.box === box;
  state.target = target;
  state.box = box;
  if (!stable) return pending();
  if (['click', 'hover', 'tap'].includes(command.input.action)) {
    const hit = target.ownerDocument.elementFromPoint(
      rect.x + rect.width / 2,
      rect.y + rect.height / 2,
    );
    if (!hit || !(hit === target || target.contains(hit))) return pending();
  }
  return { done: true, target, value: null };
}
