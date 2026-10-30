import { submitterActionCases } from './submitter-action-cases.mjs';
export const inputSourceCases = submitterActionCases;
export function measureInputSources(cases) {
  return cases.map(({ base, value, kind }) => {
    const owner = document.implementation.createHTMLDocument(),
      input = owner.createElement('input');
    if (base !== null) {
      const b = owner.createElement('base');
      b.setAttribute('href', base);
      owner.head.append(b);
    }
    owner.body.append(input);
    if (value !== null) {
      if (kind === 'attribute') input.setAttribute('src', value);
      else input.src = value;
    }
    return [input.src, input.getAttribute('src')];
  });
}
const values = [
  '',
  '0',
  '1',
  '-1',
  '+1',
  ' 12px',
  '1.5',
  '01',
  '2147483647',
  '2147483648',
  '4294967295',
  '4294967296',
  '-0',
  '1e2',
  'Infinity',
  'NaN',
  '💡',
  '\ud800',
];
export const inputDimensionCases = ['text', 'image', 'file', 'IMAGE', 'unknown'].flatMap((type) =>
  ['width', 'height'].flatMap((name) =>
    ['attribute', 'property'].flatMap((kind) =>
      values.map((value) => ({ type, name, kind, value })),
    ),
  ),
);
export function measureInputDimensions(cases) {
  return cases.map(({ type, name, kind, value }) => {
    const i = document.createElement('input');
    i.type = type;
    let error = null;
    try {
      if (kind === 'attribute') i.setAttribute(name, value);
      else i[name] = value;
    } catch (e) {
      error = e.name;
    }
    return [error, i[name], i.getAttribute(name)];
  });
}
