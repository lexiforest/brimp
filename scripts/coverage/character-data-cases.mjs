const values = [
  'undefined',
  'null',
  'false',
  '0',
  '1n',
  'Symbol()',
  ...['', 'x', '\0\ud800X\udc00', '\ud83d\ude00'].map(JSON.stringify),
  '({[Symbol.toPrimitive](hint){return hint}})',
  '({toString(){throw new RangeError("conversion")}})',
];
const offsets = [
  'undefined',
  'null',
  'false',
  '0',
  '1',
  '2',
  '4',
  '5',
  '99',
  '-1',
  '1.9',
  'NaN',
  'Infinity',
  '4294967297',
  '1n',
  'Symbol()',
];
const calls = [
  ...values.map((v) => ({ method: 'data', args: [v] })),
  ...values.map((v) => ({ method: 'appendData', args: [v] })),
  ...offsets.flatMap((offset) =>
    ['0', '2', '99', '-1', 'undefined', '1n', 'Symbol()'].flatMap((count) => [
      { method: 'substringData', args: [offset, count] },
      { method: 'deleteData', args: [offset, count] },
    ]),
  ),
  ...offsets.flatMap((offset) =>
    ['"x"', 'null', 'Symbol()'].map((value) => ({ method: 'insertData', args: [offset, value] })),
  ),
  ...offsets.flatMap((offset) =>
    ['0', '2', '99', '1n'].flatMap((count) =>
      ['"xy"', 'null', 'Symbol()'].map((value) => ({
        method: 'replaceData',
        args: [offset, count, value],
      })),
    ),
  ),
];
export const characterDataCases = ['text', 'comment'].flatMap((kind) =>
  ['', 'abcd', 'A\ud83d\ude00\0\ud800\udc00Z'].flatMap((initial) =>
    calls.map((call) => ({ kind, initial, ...call })),
  ),
);
export function measureCharacterData(cases) {
  return cases.map((c) => {
    const d = document.implementation.createHTMLDocument(),
      node = c.kind === 'text' ? d.createTextNode(c.initial) : d.createComment(c.initial);
    d.body.appendChild(node);
    const ranges = Array.from({ length: c.initial.length + 1 }, (_, offset) => {
      const r = d.createRange();
      r.setStart(node, offset);
      r.collapse(true);
      return r;
    });
    const observer = new MutationObserver(() => {});
    observer.observe(node, { characterData: true, characterDataOldValue: true });
    let result,
      error = null;
    try {
      const args = c.args.map((source) => Function('return (' + source + ')')());
      if (c.method === 'data') {
        node.data = args[0];
        result = 'undefined';
      } else {
        const value = node[c.method](...args);
        result = value === undefined ? 'undefined' : value;
      }
    } catch (e) {
      error = e.name;
    }
    const records = observer.takeRecords().map((r) => [r.type, r.target === node, r.oldValue]);
    observer.disconnect();
    return {
      result: result ?? null,
      error,
      data: node.data,
      length: node.length,
      ranges: ranges.map((r) => [
        r.startContainer === node,
        r.startOffset,
        r.endContainer === node,
        r.endOffset,
      ]),
      records,
    };
  });
}
