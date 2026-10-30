export const controlSelectionCases = [];
const types = [
  'text',
  'search',
  'url',
  'tel',
  'password',
  'email',
  'number',
  'date',
  'hidden',
  'checkbox',
  'file',
  'range',
  'color',
  'month',
  'week',
  'textarea',
];
const operations = [
  ['select'],
  ['setSelectionRange', 1, 3],
  ['setSelectionRange', 4, 2, 'backward'],
  ['setSelectionRange', -1, 99, 'forward'],
  ['setSelectionRange', 1, 3, 'BACKWARD'],
  ['setSelectionRange', 1, 1, 'backward'],
  ['selectionStart', -1],
  ['selectionEnd', -1],
  ['selectionStart', 2.9],
  ['selectionEnd', 4294967297],
  ['selectionStart', null],
  ['selectionEnd', null],
  ['selectionDirection', 'forward'],
  ['selectionDirection', 'BACKWARD'],
  ['setRangeText', 'XY'],
  ['setRangeText', 'XY', 2, 4, 'select'],
  ['setRangeText', 'x', 4, 2, 'preserve'],
  ['setRangeText', 'x', -1, 99, 'preserve'],
  ['setRangeText', 'x', 2, 99, 'bad'],
];
for (const type of types)
  for (const operation of operations) controlSelectionCases.push({ type, operation });
for (const type of ['text', 'url', 'textarea'])
  for (const replacement of ['XY', '', 'x\r\ny', '😀'])
    for (const mode of ['select', 'start', 'end', 'preserve', 'bad', 'SELECT', null])
      for (const [start, end] of [
        [0, 0],
        [2, 4],
        [0, 99],
        [99, 100],
      ])
        controlSelectionCases.push({
          type,
          operation: ['setRangeText', replacement, start, end, mode],
        });

export function measureControlSelections(cases) {
  return cases.map(({ type, operation }) => {
    const i = document.createElement(type === 'textarea' ? 'textarea' : 'input');
    if (type !== 'textarea') i.type = type;
    if (type !== 'file')
      i.value =
        type === 'color' ? '#123456' : type === 'number' || type === 'range' ? '123456' : 'abcdef';
    if (i.selectionStart !== null) i.setSelectionRange(1, 4, 'backward');
    let error = null;
    try {
      const [name, ...args] = operation;
      if (name.startsWith('selection')) i[name] = args[0];
      else i[name](...args);
    } catch (e) {
      error = e.name;
    }
    return [error, i.value, i.selectionStart, i.selectionEnd, i.selectionDirection];
  });
}
