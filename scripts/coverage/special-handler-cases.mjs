const returns = [
  'undefined',
  'null',
  'false',
  'true',
  '0',
  '1',
  '""',
  '"message"',
  'new Boolean(true)',
  '({toString(){conversions.push("string");return "converted"}})',
];
export const specialHandlerCases = [
  ...['window', 'body', 'frameset', 'div', 'iframe'].flatMap((target) =>
    ['Event', 'ErrorEvent'].flatMap((kind) =>
      returns.flatMap((source) =>
        [false, true].map((cancelable) => ({
          type: 'error',
          target,
          kind,
          source,
          cancelable,
          initial: '',
        })),
      ),
    ),
  ),
  ...['window', 'body', 'frameset', 'iframe'].flatMap((target) =>
    ['Event', 'BeforeUnloadEvent'].flatMap((kind) =>
      returns.flatMap((source) =>
        [false, true].flatMap((cancelable) =>
          ['', 'preset'].map((initial) => ({
            type: 'beforeunload',
            target,
            kind,
            source,
            cancelable,
            initial,
          })),
        ),
      ),
    ),
  ),
];
export function measureSpecialHandlers(cases) {
  const frame = document.createElement('iframe');
  document.body.append(frame);
  try {
    return cases.map(({ type, target, kind, source, cancelable, initial }) => {
      const receiver =
        target === 'window'
          ? window
          : target === 'iframe'
            ? frame.contentWindow
            : document.createElement(target);
      const eventTarget = target === 'body' || target === 'frameset' ? window : receiver,
        property = 'on' + type,
        prior = receiver[property];
      const payload = {},
        conversions = [],
        calls = [];
      const event =
        kind === 'BeforeUnloadEvent'
          ? document.createEvent(kind)
          : new window[kind](type, {
              cancelable,
              message: 'message',
              filename: 'source.js',
              lineno: 3,
              colno: 4,
              error: payload,
            });
      if (kind === 'BeforeUnloadEvent') {
        event.initEvent(type, false, cancelable);
        event.returnValue = initial;
      }
      try {
        receiver[property] = function (...args) {
          calls.push([
            this === eventTarget,
            args.length,
            args[0] === event ? 'event' : args[0],
            args[1] ?? null,
            args[2] ?? null,
            args[3] ?? null,
            args[4] === payload,
          ]);
          return Function('conversions', 'return (' + source + ')')(conversions);
        };
        const result = eventTarget.dispatchEvent(event);
        return {
          calls,
          conversions,
          result,
          prevented: event.defaultPrevented,
          returnValue: event.returnValue,
        };
      } finally {
        receiver[property] = prior;
      }
    });
  } finally {
    frame.remove();
  }
}
