// Inputs are independent of the runtime recognizer. Expected modes are measured
// in installed Safari and stored in the test fixture, never inferred here.
export function documentModeCases() {
  const cases = [];
  const add = (id, html) => cases.push({ id, html });
  const voids = new Set(['br', 'input', 'img', 'hr', 'meta', 'link']);
  const element = (tag, body = 'x') => `<${tag}>${voids.has(tag) ? '' : body + `</${tag}>`}`;
  const supported = [
    'a',
    'b',
    'br',
    'button',
    'div',
    'footer',
    'i',
    'input',
    'li',
    'label',
    'option',
    'ol',
    'p',
    'select',
    'span',
    'strong',
    'ul',
  ];
  for (const tag of [
    ...supported,
    'body',
    'html',
    'head',
    'title',
    'script',
    'style',
    'svg',
    'math',
    'table',
    'tr',
    'td',
    'img',
    'hr',
    'meta',
    'link',
    'textarea',
    'form',
    'fieldset',
    'section',
    'header',
    'main',
    'nav',
    'article',
    'h1',
    'em',
    'u',
    's',
    'small',
    'pre',
    'code',
    'dl',
    'dt',
    'dd',
    'datalist',
    'optgroup',
    'video',
    'audio',
    'canvas',
    'template',
    'x-custom',
  ]) {
    add(`tag:${tag}`, element(tag));
    add(`tag:upper:${tag}`, element(tag).replaceAll(tag, tag.toUpperCase()));
  }
  for (const parent of supported.filter((t) => !voids.has(t)))
    for (const child of supported) {
      add(`nest:${parent}:${child}`, element(parent, element(child)));
    }
  for (const name of [
    'id',
    'class',
    'data-x',
    'data-X',
    'onclick',
    'ONCLICK',
    'on',
    'one',
    'on_',
    'is',
    'dir',
    'style',
    'type',
    'disabled',
    '0',
    '_',
    '-',
    'x:y',
    'x.y',
    'é',
  ]) {
    for (const value of ['', '=x', "='x'", '="&quot;x\r\ny"'])
      add(`attribute:${name}:${value}`, `<div ${name}${value}></div>`);
  }
  for (const attributes of [
    'x=',
    'x= x',
    'x=x/',
    'x=x /',
    'x=x:y',
    'x=a.b',
    'x=&amp;',
    'x=é',
    'x=""y="z"',
    'x="a<b"',
    'x="a>b"',
    "x='a\"b'",
    'x="a" x="b"',
    'X=a x=b',
    'a / ',
    '/ ',
    '/ x',
    'x',
    "x='unterminated",
    'x==',
    'x=`a`',
    'x="\0"',
    "x='&#0;'",
  ]) {
    add(`attribute-syntax:${attributes}`, `<div ${attributes}></div>`);
  }
  for (const name of ['onclick', 'onerror', 'on', 'is', 'id', 'data-λ', 'é', 'x:y']) {
    for (const suffix of ['', ' ', '/', ' /', '/ ', ' = "x"', '="x"', '="x"/', '\t'])
      add(`attribute-ending:${name}:${suffix}`, `<div ${name}${suffix}></div>`);
  }
  for (const html of [
    '',
    ' ',
    '\t\r\n\f',
    'x',
    'é',
    '😀',
    'a\r\nb',
    '\0',
    '&#0;',
    '&amp;',
    '&bogus;',
    '&notit;',
    '&lt;div&gt;',
    '&#x1F600;',
    '&',
    '<',
    '</div>',
    '<div>',
    '<div/>',
    '<div/></div>',
    '<br/>',
    '<br / >',
    '<input x=y/>',
    '<input x=>',
    '<p>x</P>',
    '<DIV>x</div>',
    '<div>x</div >',
    '<div>x</div/>',
    '<div>x</div x>',
    '<div>x</ div>',
    '<p><div>x</div></p>',
    '<li><ul><li>x</li></ul></li>',
    '<ul><li><li>x</li></li></ul>',
    '<a><div><a>x</a></div></a>',
    '<p><a><div>x</div></a></p>',
    '<ul>x<li>y</li></ul>',
    '<select>x<option>y</option></select>',
    '<!--x--><p>x</p>',
    '<?xml x?>',
    '<!doctype html>',
    '<!doctype potato>',
    '<!doctype html><p>x</p>',
    '<html><body>x</body></html>',
  ])
    add(`literal:${JSON.stringify(html)}`, html);
  for (const length of [65535, 65536, 65537]) {
    for (const [name, text] of [
      ['ascii', 'x'.repeat(length)],
      ['entities', '&amp;'.repeat(length)],
      ['crlf', '\r\n'.repeat(length)],
      ['utf16', '😀'.repeat(Math.floor(length / 2)) + (length % 2 ? 'x' : '')],
      ['whitespace', '\n' + ' '.repeat(length)],
    ])
      add(`text-limit:${name}:${length}`, `<div>${text}</div>`);
  }
  for (const depth of [510, 511, 512, 513])
    add(`depth:${depth}`, '<div>'.repeat(depth) + 'x' + '</div>'.repeat(depth));
  return cases;
}
