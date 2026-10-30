export const labelAssociationCases = [
  'detached',
  'connected',
  'fragment',
  'template',
  'shadow',
  'shadow-detached',
  'parsed',
].flatMap((context) =>
  ['input', 'button', 'select', 'textarea', 'output', 'meter', 'progress'].flatMap((tag) =>
    [
      'implicit',
      'explicit',
      'empty-for',
      'missing-for',
      'duplicate-id',
      'nested-label',
      'hidden-first',
      'foreign-first',
      'foreign-for',
      'root-label',
      'move',
      'type-change',
    ].map((operation) => ({ context, tag, operation })),
  ),
);
export function measureLabelAssociations(cases) {
  return cases.map(({ context, tag, operation }) => {
    let root, cleanup;
    if (context === 'parsed') root = new DOMParser().parseFromString('', 'text/html').body;
    else if (context === 'fragment') root = document.createDocumentFragment();
    else if (context === 'template') root = document.createElement('template').content;
    else if (context.startsWith('shadow')) {
      cleanup = document.createElement('div');
      if (context === 'shadow') document.body.append(cleanup);
      root = cleanup.attachShadow({ mode: 'open' });
    } else {
      root = document.createElement('div');
      if (context === 'connected') {
        document.body.append(root);
        cleanup = root;
      }
    }
    const f = document.createElement('form'),
      label = document.createElement('label'),
      external = document.createElement('label'),
      control = document.createElement(tag);
    f.id = 'form';
    label.id = 'label';
    external.id = 'external';
    control.id = 'control';
    root.append(f, external);
    f.append(label);
    label.append(control);
    external.htmlFor = 'control';
    if (operation === 'explicit') label.htmlFor = 'control';
    if (operation === 'empty-for') label.htmlFor = '';
    if (operation === 'missing-for') label.htmlFor = 'missing';
    if (operation === 'duplicate-id') {
      label.htmlFor = 'control';
      const div = document.createElement('div');
      div.id = 'control';
      root.prepend(div);
    }
    if (operation === 'nested-label') {
      const nested = document.createElement('label');
      nested.id = 'nested';
      label.prepend(nested);
      nested.append(control);
    }
    if (operation === 'hidden-first') {
      const other = document.createElement('input');
      other.type = 'hidden';
      label.prepend(other);
    }
    if (operation === 'foreign-first') {
      const other = document.createElementNS('urn:other', 'input');
      label.prepend(other);
    }
    if (operation === 'foreign-for') label.setAttributeNS('urn:other', 'for', 'missing');
    if (operation === 'root-label') {
      root = label;
      label.remove();
    }
    const list = control.labels;
    const snapshot = () => [
      label.control === control,
      label.form === f,
      control.labels === null ? null : Array.from(control.labels, (l) => l.id),
      list === control.labels,
    ];
    const result = [snapshot()];
    if (operation === 'move') {
      root.prepend(control);
    } else if (operation === 'type-change') {
      control.setAttribute('type', 'hidden');
    } else control.id = 'changed';
    result.push(snapshot(), list === null ? null : Array.from(list, (l) => l.id));
    control.id = 'control';
    control.removeAttribute('type');
    result.push(snapshot());
    cleanup?.remove();
    return result;
  });
}
