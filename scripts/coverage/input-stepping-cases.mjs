const scenarios = {
  number: [
    {},
    { value: '0.3', step: '0.1' },
    { value: '0.30000000000000004', step: '0.1' },
    { value: '3', min: '-100', step: '3' },
    { value: '3', min: '10', max: '20', step: '3' },
    { value: '22', min: '10', max: '20', step: '3' },
    { value: '0', max: '-10', step: '3' },
    { value: '4', min: '10', max: '5' },
    { value: '1.5', defaultValue: '0.2', step: '1' },
    { value: '5', min: '1', max: '2', step: '10' },
    { value: '-0' },
    { value: '1e21', step: '0.1' },
    { value: '1.2345678901234567', step: '0.000000000000001' },
    { value: '1e100', step: '1e80' },
    { value: '1e-100', step: '1e-100' },
    { value: '1.7976931348623157e308', step: '1e308' },
    { value: '5e-324', step: '5e-324' },
    { value: '', min: '1.1', max: '1.2', step: '10' },
    { value: '3.1', step: '2' },
    { value: '-2', step: '3' },
    { value: '-1', step: '10' },
    { value: '-0.01', step: '10' },
    { value: '10.1' },
    { value: '-10.1' },
    { value: '10.01' },
    { value: '-10.01' },
  ],
  range: [
    {},
    { value: '0.3', min: '-1', max: '1', step: '0.1' },
    { value: '5', min: '10', max: '2' },
    { value: '1', defaultValue: '0.3', step: '3' },
    { min: '-10', max: '-1', step: '4' },
    { min: '2', max: '3', step: '10' },
  ],
  date: [
    {},
    { value: '2000-02-28' },
    { value: '1900-02-28' },
    { value: '0001-01-01' },
    { value: '275760-09-13' },
    { value: '1970-01-02', step: '2' },
    { value: '1969-12-31', defaultValue: '1970-01-02', step: '3' },
    { min: '2000-01-01', max: '2000-01-10', step: '3' },
    { max: '1960-01-01' },
    { value: '2000-01-02', min: '2000-01-05', max: '2000-01-01' },
    { value: '2000-01-01', step: '0.5' },
    { value: '2000-01-01', step: '1.5' },
    { value: '000099-12-31', step: '2' },
  ],
  time: [
    {},
    { value: '23:59:59.999', step: '0.001' },
    { value: '00:00:00.001', step: '0.001' },
    { value: '12:34:56.789' },
    { value: '12:34:56.789', step: '0.1' },
    { min: '13:00', max: '14:00', step: '90' },
    { max: '00:00:10', step: '3' },
    { value: '23:00', min: '22:00', max: '02:00' },
    { value: '00:00:00.003', defaultValue: '00:00:00.001', step: '0.002' },
    { value: '00:00', step: '0.0001' },
    { value: '12:00', step: '86400' },
    { value: '12:00', min: '12:00:00.001', max: '12:00:00.002', step: '0.01' },
  ],
  'datetime-local': [
    {},
    { value: '2000-02-28T23:59' },
    { value: '0001-01-01T00:00' },
    { value: '275760-09-13T00:00' },
    { value: '1969-12-31T23:59:59.999', step: '0.001' },
    { value: '2000-01-01T12:34:56.789', step: '0.1' },
    { min: '2000-01-01T00:00', max: '2000-01-01T00:02', step: '90' },
    { value: '1970-01-01T00:00:00.003', defaultValue: '1970-01-01T00:00:00.001', step: '0.002' },
  ],
  text: [{ value: 'hello' }],
  month: [{ value: '2000-01' }],
  week: [{ value: '2000-W01' }],
  file: [{}],
  checkbox: [{}],
};
const counts = [
  'default',
  'undefined',
  'null',
  '0',
  '1',
  '-1',
  '2',
  '-2',
  '1.9',
  '-1.9',
  'NaN',
  'Infinity',
  '2147483648',
  '4294967297',
  'bigint',
  'symbol',
  'object',
];
export const inputSteppingCases = Object.entries(scenarios).flatMap(([type, rows]) =>
  rows.flatMap((row) =>
    ['stepUp', 'stepDown'].flatMap((method) =>
      counts.map((count) => ({ type, ...row, method, count })),
    ),
  ),
);
for (const type of ['number', 'range', 'date', 'time', 'datetime-local'])
  for (const step of [
    'any',
    'ANY',
    'Any',
    ' any',
    '0',
    '-1',
    'invalid',
    '+2',
    ' 2',
    '1e100',
    '1e-100',
    '1.0000000000000001',
  ])
    for (const method of ['stepUp', 'stepDown'])
      inputSteppingCases.push({ type, step, method, count: 'default' });
export function measureInputStepping(cases) {
  return cases.map(({ type, value, defaultValue, min, max, step, method, count }) => {
    const input = document.createElement('input');
    input.type = type;
    for (const [name, v] of Object.entries({ min, max, step }))
      if (v !== undefined) input.setAttribute(name, v);
    if (defaultValue !== undefined) input.defaultValue = defaultValue;
    if (value !== undefined) input.value = value;
    const before = input.value,
      log = [];
    let n;
    if (count === 'null') n = null;
    else if (count === 'bigint') n = 1n;
    else if (count === 'symbol') n = Symbol();
    else if (count === 'object')
      n = {
        [Symbol.toPrimitive](hint) {
          log.push(hint);
          return 2;
        },
      };
    else if (count !== 'undefined' && count !== 'default') n = Number(count);
    let error = null,
      result;
    try {
      result = count === 'default' ? input[method]() : input[method](n);
    } catch (e) {
      error = e.name;
    }
    const after = input.value;
    // A no-op and a write of the same string differ when the default changes.
    input.defaultValue =
      type === 'date'
        ? '2001-01-01'
        : type === 'time'
          ? '01:02'
          : type === 'datetime-local'
            ? '2001-01-01T01:02'
            : type === 'file'
              ? ''
              : '19';
    return [before, error, after, input.value, typeof result, log];
  });
}
