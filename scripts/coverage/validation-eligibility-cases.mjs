const inputs = [
  'text',
  'search',
  'tel',
  'url',
  'email',
  'password',
  'number',
  'range',
  'color',
  'date',
  'time',
  'datetime-local',
  'month',
  'week',
  'file',
  'checkbox',
  'radio',
  'hidden',
  'submit',
  'image',
  'reset',
  'button',
  'invalid',
];
const controls = [
  ...inputs.map((type) => ({ tag: 'input', type })),
  ...['submit', 'reset', 'button', 'auto'].map((type) => ({ tag: 'button', type })),
  ...['select', 'textarea', 'fieldset', 'output', 'object'].map((tag) => ({ tag })),
];
const contexts = [
  'detached',
  'connected',
  'inactive-document',
  'novalidate-form',
  'datalist',
  'foreign-datalist',
  'disabled-fieldset',
  'first-legend',
  'second-legend',
  'preceded-legend',
  'foreign-legend',
  'nested-disabled',
  'outer-disabled',
  'shadow-fieldset',
  'shadow-datalist',
];
const attributes = [
  [],
  ['disabled'],
  ['readonly'],
  ['disabled', 'readonly'],
  ['hidden', 'inert'],
  ['formnovalidate'],
  ['command'],
];
export const validationEligibilityCases = controls.flatMap((control) =>
  contexts.flatMap((context) =>
    attributes.map((attributes) => ({ ...control, context, attributes })),
  ),
);
export function measureValidationEligibility(cases) {
  return cases.map(({ tag, type, context, attributes }) => {
    const owner =
      context === 'inactive-document' ? document.implementation.createHTMLDocument() : document;
    const control = owner.createElement(tag),
      container = owner.createElement('div');
    if (type !== undefined) control.setAttribute('type', type);
    for (const attribute of attributes) control.setAttribute(attribute, '');
    container.append(control);
    if (context === 'novalidate-form') {
      const form = owner.createElement('form');
      form.noValidate = true;
      container.append(form);
      form.append(control);
    }
    if (context === 'datalist' || context === 'foreign-datalist') {
      const parent =
        context === 'datalist'
          ? owner.createElement('datalist')
          : owner.createElementNS('http://www.w3.org/2000/svg', 'datalist');
      container.append(parent);
      parent.append(control);
    }
    if (
      [
        'disabled-fieldset',
        'first-legend',
        'second-legend',
        'preceded-legend',
        'foreign-legend',
        'nested-disabled',
        'outer-disabled',
      ].includes(context)
    ) {
      const fieldset = owner.createElement('fieldset');
      fieldset.disabled = true;
      container.append(fieldset);
      fieldset.append(control);
      if (context !== 'disabled-fieldset') {
        const legend =
          context === 'foreign-legend'
            ? owner.createElementNS('http://www.w3.org/2000/svg', 'legend')
            : owner.createElement('legend');
        if (context === 'second-legend') fieldset.prepend(owner.createElement('legend'));
        if (context === 'preceded-legend') fieldset.prepend(owner.createElement('div'));
        fieldset.append(legend);
        legend.append(control);
        if (context === 'nested-disabled') {
          const inner = owner.createElement('fieldset');
          inner.disabled = true;
          legend.append(inner);
          inner.append(control);
        }
        if (context === 'outer-disabled') {
          const outer = owner.createElement('fieldset');
          outer.disabled = true;
          container.append(outer);
          outer.append(fieldset);
        }
      }
    }
    if (context === 'shadow-fieldset' || context === 'shadow-datalist') {
      const parent = owner.createElement(context === 'shadow-fieldset' ? 'fieldset' : 'datalist'),
        host = owner.createElement('div');
      parent.setAttribute('disabled', '');
      container.append(parent);
      parent.append(host);
      host.attachShadow({ mode: 'open' }).append(control);
    }
    if (context === 'connected') owner.body.append(container);
    const result = control.willValidate;
    container.remove();
    return result;
  });
}

const sequences = [
  [
    'inner-enable',
    'inner-disable',
    'outer-enable',
    'outer-disable',
    'inner-enable',
    'reinsert-control',
  ],
  [
    'prepend-legend',
    'inner-disable',
    'remove-extra-legend',
    'outer-disable',
    'inner-enable',
    'append-text',
    'edit-text',
  ],
  [
    'inner-disable',
    'outer-disable',
    'inner-enable',
    'clone',
    'detach-inner',
    'restore-inner',
    'adopt-control',
    'restore-control',
  ],
  [
    'inner-disable',
    'outer-disable',
    'inner-enable',
    'detach-outer',
    'restore-outer',
    'detach-outer',
    'clone',
  ],
];
export const validationEligibilityMutationCases = ['input', 'button', 'select', 'textarea'].flatMap(
  (tag) =>
    [false, true].flatMap((outerDisabled) =>
      [false, true].flatMap((innerDisabled) =>
        [false, true].flatMap((inLegend) =>
          [false, true].flatMap((connected) =>
            sequences.map((operations) => ({
              tag,
              outerDisabled,
              innerDisabled,
              inLegend,
              connected,
              operations,
            })),
          ),
        ),
      ),
    ),
);
export function measureValidationEligibilityMutations(cases) {
  return cases.map(({ tag, outerDisabled, innerDisabled, inLegend, connected, operations }) => {
    const outer = document.createElement('fieldset'),
      inner = document.createElement('fieldset'),
      legend = document.createElement('legend'),
      extra = document.createElement('legend');
    const control = document.createElement(tag),
      text = document.createTextNode('a');
    outer.disabled = outerDisabled;
    inner.disabled = innerDisabled;
    inner.append(legend);
    (inLegend ? legend : inner).append(control);
    outer.append(inner);
    if (connected) document.body.append(outer);
    const result = [control.willValidate];
    for (const operation of operations) {
      if (operation === 'inner-enable') inner.disabled = false;
      if (operation === 'inner-disable') inner.disabled = true;
      if (operation === 'outer-enable') outer.disabled = false;
      if (operation === 'outer-disable') outer.disabled = true;
      if (operation === 'reinsert-control') control.parentNode.append(control);
      if (operation === 'prepend-legend') inner.prepend(extra);
      if (operation === 'remove-extra-legend') extra.remove();
      if (operation === 'append-text') inner.append(text);
      if (operation === 'edit-text') text.data = 'b';
      if (operation === 'clone') {
        const clone = outer.cloneNode(true);
        result.push(clone.querySelector(tag).willValidate);
        continue;
      }
      if (operation === 'detach-inner') inner.remove();
      if (operation === 'restore-inner') outer.append(inner);
      if (operation === 'detach-outer') outer.remove();
      if (operation === 'restore-outer') document.body.append(outer);
      if (operation === 'adopt-control')
        document.implementation.createHTMLDocument().adoptNode(control);
      if (operation === 'restore-control') (inLegend ? legend : inner).append(control);
      result.push(control.willValidate);
    }
    outer.remove();
    return result;
  });
}
