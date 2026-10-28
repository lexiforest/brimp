// Browser consoles expose far more than the five logging level methods this
// runtime creates natively. Scripts and probes use the rest for grouping,
// timing and counting, so provide the standard surface on top of `console.log`.
(function (global) {
  const target = global.console;
  if (!target) return;
  const now = () =>
    global.performance && typeof global.performance.now === 'function'
      ? global.performance.now()
      : Date.now();
  const counters = new Map();
  const timers = new Map();
  const define = (name, value) => {
    if (typeof target[name] === 'function') return;
    Object.defineProperty(target, name, {
      value,
      writable: true,
      enumerable: true,
      configurable: true,
    });
  };
  const label = (value) => (value === undefined ? 'default' : String(value));
  define('assert', function assert(condition, ...rest) {
    if (condition) return;
    if (rest.length === 0) target.error('Assertion failed');
    else target.error('Assertion failed:', ...rest);
  });
  define('clear', function clear() {});
  define('count', function count(name) {
    const key = label(name);
    const next = (counters.get(key) ?? 0) + 1;
    counters.set(key, next);
    target.log(`${key}: ${next}`);
  });
  define('countReset', function countReset(name) {
    counters.delete(label(name));
  });
  define('dir', function dir(...arguments_) {
    target.log(...arguments_);
  });
  define('dirxml', function dirxml(...arguments_) {
    target.log(...arguments_);
  });
  define('group', function group(...arguments_) {
    if (arguments_.length !== 0) target.log(...arguments_);
  });
  define('groupCollapsed', function groupCollapsed(...arguments_) {
    if (arguments_.length !== 0) target.log(...arguments_);
  });
  define('groupEnd', function groupEnd() {});
  define('table', function table(...arguments_) {
    if (arguments_.length !== 0) target.log(...arguments_);
  });
  define('time', function time(name) {
    timers.set(label(name), now());
  });
  define('timeLog', function timeLog(name, ...rest) {
    const key = label(name);
    const start = timers.get(key);
    if (start === undefined) return;
    target.log(`${key}: ${now() - start} ms`, ...rest);
  });
  define('timeEnd', function timeEnd(name) {
    const key = label(name);
    const start = timers.get(key);
    if (start === undefined) return;
    timers.delete(key);
    target.log(`${key}: ${now() - start} ms`);
  });
  define('trace', function trace(...arguments_) {
    if (arguments_.length !== 0) target.log('Trace:', ...arguments_);
  });
  define('profile', function profile() {});
  define('profileEnd', function profileEnd() {});
  define('timeStamp', function timeStamp() {});
  // WebKit adds four console members that Blink does not have; follow the
  // profile identity instead of exposing them everywhere.
  const webkit = !/Chrome\//.test(global.navigator?.userAgent ?? '');
  if (webkit) {
    define('takeHeapSnapshot', function takeHeapSnapshot() {});
    define('record', function record() {});
    define('recordEnd', function recordEnd() {});
    define('screenshot', function screenshot() {});
  }
})(globalThis);
