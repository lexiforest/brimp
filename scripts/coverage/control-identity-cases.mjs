export const controlIdentityCases = [
  ['input', 'HTMLInputElement'],
  ['button', 'HTMLButtonElement'],
  ['a', 'HTMLAnchorElement'],
];
export function measureControlIdentity(cases) {
  return cases.map(([tag, name], index) => {
    const C = globalThis[name],
      p = C.prototype,
      i = document.createElement(tag),
      constructor = Object.getOwnPropertyDescriptor(p, 'constructor'),
      prototype = Object.getOwnPropertyDescriptor(C, 'prototype'),
      brand = Object.getOwnPropertyDescriptor(p, Symbol.toStringTag);
    const shape = [
      C.name,
      C.length,
      Object.getPrototypeOf(C) === HTMLElement,
      Object.getPrototypeOf(p) === HTMLElement.prototype,
      Object.getPrototypeOf(i) === p,
      i.constructor === C,
      [prototype.value === p, prototype.writable, prototype.enumerable, prototype.configurable],
      [
        constructor.value === C,
        constructor.writable,
        constructor.enumerable,
        constructor.configurable,
      ],
      [brand.value, brand.writable, brand.enumerable, brand.configurable],
      Object.prototype.toString.call(i),
    ];
    const construction = [];
    for (const create of [
      () => C(),
      () => new C(),
      () => new (class extends C {})(),
      () => Reflect.construct(C, [], function Fake() {}),
    ])
      try {
        create();
        construction.push('ok');
      } catch (e) {
        construction.push(e.name);
      }
    class X extends C {}
    let registered;
    try {
      customElements.define('identity-built-in-' + index, X, { extends: tag });
      registered = 'ok';
    } catch (e) {
      registered = e.name;
    }
    let builtIn;
    try {
      builtIn = new X().localName;
    } catch (e) {
      builtIn = e.name;
    }
    class Y extends C {}
    customElements.define('identity-autonomous-' + index, Y);
    let autonomous;
    try {
      autonomous = new Y().localName;
    } catch (e) {
      autonomous = e.name;
    }
    return { shape, construction, registered, builtIn, autonomous };
  });
}
