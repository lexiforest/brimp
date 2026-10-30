const actions = [
  'none',
  'cancel',
  'self-false',
  'peer-true',
  'rename-cancel',
  'peer-remove-cancel',
  'checkbox',
  'checkbox-cancel',
  'text',
  'text-cancel',
  'remove',
  'reset',
  'reset-cancel',
];
export const inputActivationCases = [false, true].flatMap((connected) =>
  [false, true].flatMap((checked) =>
    ['click', 'Event', 'MouseEvent', 'PointerEvent'].flatMap((event) =>
      actions.map((action) => ({ connected, checked, event, action })),
    ),
  ),
);
export function measureInputActivation(cases) {
  return cases.map(({ connected, checked, event, action }) => {
    const form = document.createElement('form');
    if (connected) document.body.append(form);
    form.innerHTML = '<input type=radio name=g checked><input type=radio name=g>';
    const [a, b] = form.children;
    if (checked) b.checked = true;
    b.indeterminate = true;
    const log = [];
    for (const name of ['input', 'change'])
      b.addEventListener(name, (e) =>
        log.push([name, a.checked, b.checked, b.indeterminate, e.isTrusted, e.composed]),
      );
    b.addEventListener('click', (e) => {
      log.push(['click', a.checked, b.checked, b.indeterminate, e.isTrusted, e.composed]);
      if (action === 'self-false') b.checked = false;
      if (action === 'peer-true') a.checked = true;
      if (action === 'rename-cancel') b.name = 'other';
      if (action === 'peer-remove-cancel') a.remove();
      if (action.startsWith('checkbox')) b.type = 'checkbox';
      if (action.startsWith('text')) b.type = 'text';
      if (action === 'remove') b.remove();
      if (action.startsWith('reset')) form.reset();
      if (action.includes('cancel')) e.preventDefault();
    });
    if (event === 'click') b.click();
    else b.dispatchEvent(new globalThis[event]('click', { bubbles: true, cancelable: true }));
    const result = [a.checked, b.checked, b.indeterminate, log];
    form.remove();
    return result;
  });
}
