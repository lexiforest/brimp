export const inputValidityFlagNames = [
  'valueMissing',
  'typeMismatch',
  'patternMismatch',
  'tooLong',
  'tooShort',
  'rangeUnderflow',
  'rangeOverflow',
  'stepMismatch',
  'badInput',
  'customError',
  'valid',
];
const types = [
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
];
export const inputValidityCases = types.flatMap((type) =>
  (type === 'file'
    ? ['']
    : ['', 'x', '1', 'a@b', 'https://example.test/', '2000-01-01', '12:00']
  ).flatMap((value) =>
    [
      {},
      { required: '' },
      { required: '', readonly: '' },
      { required: '', disabled: '' },
      { required: '', checked: '' },
      { minlength: '3', maxlength: '5' },
      { pattern: '[a-z]+' },
    ].map((attributes) => ({ type, value: type === 'file' ? '' : value, attributes })),
  ),
);
for (const type of ['email', 'url'])
  for (const value of [
    'a',
    'a@b',
    'a@b.c',
    '.a@b',
    'a..b@c',
    'a@-b',
    'a@b-',
    'a@b..c',
    'a@b_c',
    'a@b.',
    'a+b@c',
    'a b@c',
    'a@例子.test',
    'é@b',
    'a@xn--fsqu00a.test',
    'a@b,c@d',
    'a@b,',
    'a@b,,c@d',
    ' , ',
    'http://',
    'http://[bad',
    'relative/path',
    '//example.test',
    'http://example.test',
    'HTTP://EXAMPLE.test',
    'https://例子.test',
    'javascript:x',
    'mailto:x',
    'data:,',
    'about:blank',
    'a:b',
    'http://ex ample.test',
    'https://example.test/\ud800',
    'a\u0000b',
  ])
    for (const attributes of [
      {},
      { multiple: '' },
      { readonly: '' },
      { disabled: '' },
      { pattern: '.+@.+', multiple: '' },
    ])
      inputValidityCases.push({ type, value, attributes });
for (const type of ['text', 'search', 'tel', 'email', 'url', 'password', 'number', 'date'])
  for (const pattern of [
    '',
    'a',
    'a|b',
    '[a-z]+',
    '[',
    '[a-z&&[^aeiou]]+',
    '\\p{Letter}+',
    '^a$',
    'a$|b',
    '[a-z-]+',
    '.',
    '(?=a)a',
    '[\\q{ab|cd}]',
  ])
    for (const value of ['', 'a', 'b', 'ab', 'a\n', 'é', '💡', '-', 'a@b'])
      inputValidityCases.push({ type, value, attributes: { pattern } });
for (const pattern of ['.+', '[^,]+', 'a@b', '[a-z]+@b', ''])
  for (const value of ['a@b,c@d', 'a@b,a@b', ',', 'a@b,', 'a@b,,', ' , a@b', ''])
    for (const attributes of [
      { pattern, multiple: '' },
      { pattern, multiple: '', required: '' },
    ])
      inputValidityCases.push({ type: 'email', value, attributes });
const numeric = {
  number: [
    '',
    '0',
    '-0',
    '1',
    '1.1',
    '0.30000000000000004',
    '-1',
    '1e21',
    '1e-100',
    '5e-324',
    '1.2345678901234567',
    '2',
  ],
  range: ['', '0', '1', '1.1', '50', '100', '-1', '101'],
  date: ['', '1969-12-31', '1970-01-01', '1970-01-02', '2000-01-01', '2000-01-02', '275760-09-13'],
  time: ['', '00:00', '00:00:00.001', '00:00:01', '01:00', '12:00', '23:59:59.999'],
  'datetime-local': [
    '',
    '1969-12-31T23:59:59.999',
    '1970-01-01T00:00',
    '2000-01-01T12:00',
    '2000-01-01T12:00:00.001',
    '275760-09-13T00:00',
  ],
};
for (const [type, values] of Object.entries(numeric)) {
  const min =
    type === 'date'
      ? '2000-01-01'
      : type === 'time'
        ? '12:00'
        : type === 'datetime-local'
          ? '2000-01-01T12:00'
          : '1';
  const max =
    type === 'date'
      ? '2000-01-03'
      : type === 'time'
        ? '13:00'
        : type === 'datetime-local'
          ? '2000-01-01T13:00'
          : '2';
  const constraints = [
    {},
    { min },
    { max },
    { min, max },
    { min: max, max: min },
    { min: 'invalid', max: 'invalid' },
    { value: min },
    ...['any', 'ANY', '0', '-1', 'invalid', '2', '0.1', '0.0001', '1.5', '1e100', '1e-100'].map(
      (step) => ({ step }),
    ),
    { step: '2', min },
    { step: '2', value: min },
    { step: '2', min, max, readonly: '' },
    { step: '2', min, max, disabled: '' },
  ];
  for (const value of values)
    for (const attributes of constraints) inputValidityCases.push({ type, value, attributes });
}
export function measureInputValidity(cases, flags) {
  return cases.map(({ type, value, attributes }) => {
    const input = document.createElement('input');
    input.type = type;
    for (const [name, v] of Object.entries(attributes)) input.setAttribute(name, v);
    input.value = value;
    return [input.value, flags.map((name) => input.validity[name])];
  });
}

export const radioValidityCases = [
  'detached',
  'fragment',
  'form',
  'document',
  'inactive-document',
  'shadow',
  'separate-forms',
].flatMap((root) =>
  [null, '', 'g'].flatMap((name) =>
    [0, 1, 2, 3].flatMap((required) =>
      [0, 1, 2].flatMap((checked) =>
        ['none', 'self', 'peer', 'readonly-self', 'readonly-peer'].map((barred) => ({
          root,
          name,
          required,
          checked,
          barred,
        })),
      ),
    ),
  ),
);
export function measureRadioValidity(cases) {
  return cases.map(({ root, name, required, checked, barred }) => {
    const owner =
      root === 'inactive-document' ? document.implementation.createHTMLDocument() : document;
    const container = owner.createElement('div'),
      a = owner.createElement('input'),
      b = owner.createElement('input');
    for (const input of [a, b]) {
      input.type = 'radio';
      if (name !== null) input.name = name;
    }
    a.required = !!(required & 1);
    b.required = !!(required & 2);
    a.checked = checked === 1;
    b.checked = checked === 2;
    if (barred !== 'none')
      (barred.endsWith('self') ? a : b).setAttribute(
        barred.startsWith('readonly') ? 'readonly' : 'disabled',
        '',
      );
    let parent = container;
    if (root === 'fragment') parent = owner.createDocumentFragment();
    if (root === 'shadow') parent = container.attachShadow({ mode: 'open' });
    if (root === 'form') {
      parent = owner.createElement('form');
      container.append(parent);
    }
    if (root === 'separate-forms') {
      const first = owner.createElement('form'),
        second = owner.createElement('form');
      container.append(first, second);
      first.append(a);
      second.append(b);
    } else parent.append(a, b);
    if (root === 'document' || root === 'inactive-document') owner.body.append(container);
    const result = [a.validity.valueMissing, b.validity.valueMissing];
    container.remove();
    return result;
  });
}
