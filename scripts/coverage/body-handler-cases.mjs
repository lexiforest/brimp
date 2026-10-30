// Ordinary window-reflecting handlers. Error and beforeunload have distinct callback algorithms.
export const bodyWindowEventTypes = [
  'afterprint',
  'beforeprint',
  'blur',
  'focus',
  'hashchange',
  'languagechange',
  'load',
  'message',
  'messageerror',
  'offline',
  'online',
  'pagehide',
  'pageshow',
  'popstate',
  'rejectionhandled',
  'resize',
  'scroll',
  'storage',
  'unhandledrejection',
  'unload',
];
export function measureBodyHandlers(types) {
  return ['body', 'frameset'].flatMap((tag) =>
    types.map((type) => {
      const property = 'on' + type,
        e = document.createElement(tag),
        other = document.createElement(tag),
        inert = document.implementation.createHTMLDocument().createElement(tag),
        d = Object.getOwnPropertyDescriptor(Object.getPrototypeOf(e), property);
      const prior = window[property];
      window[property] = null;
      try {
        const defaults = [e[property], other[property], inert[property]];
        const assignments = [
          () => {},
          {},
          { handleEvent() {} },
          null,
          undefined,
          false,
          1,
          'source',
          Symbol(),
        ].map((value) => {
          e[property] = value;
          inert[property] = value;
          const expected =
            value !== null && (typeof value === 'object' || typeof value === 'function')
              ? value
              : null;
          return [
            e[property] === expected,
            window[property] === expected,
            other[property] === expected,
            inert[property] === null,
            Object.hasOwn(e, property),
          ];
        });
        const calls = [];
        e[property] = function (event) {
          calls.push([
            this === window,
            event.currentTarget === window,
            event.target === window,
            event.type,
          ]);
          return false;
        };
        e.dispatchEvent(new Event(type, { cancelable: true }));
        const event = new Event(type, { cancelable: true }),
          result = window.dispatchEvent(event);
        e[property] = null;
        return {
          tag,
          type,
          defaults,
          shape: d
            ? [d.enumerable, d.configurable, d.get.name, d.get.length, d.set.name, d.set.length]
            : null,
          assignments,
          calls,
          result,
          prevented: event.defaultPrevented,
        };
      } finally {
        window[property] = prior;
      }
    }),
  );
}
