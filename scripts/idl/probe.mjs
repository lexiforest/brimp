// The surface probe, run unchanged inside Safari and inside Brimp.
//
// Every descriptor becomes `[get, set, enumerable, configurable, writable,
// valueLength]`, which is what a parity check needs to compare or rebuild a
// property. `scripts/idl/capture.mjs` runs this in Safari to write the reference;
// `scripts/idl/check.mjs` runs it in Brimp and diffs the two.
export const PROBE = `(() => {
  const describe = (value) => {
    const record = {};
    for (const name of Object.getOwnPropertyNames(value)) {
      try {
        const descriptor = Object.getOwnPropertyDescriptor(value, name);
        record[name] = [
          descriptor.get ? 1 : 0,
          descriptor.set ? 1 : 0,
          descriptor.enumerable ? 1 : 0,
          descriptor.configurable ? 1 : 0,
          'value' in descriptor && descriptor.writable === true ? 1 : 0,
          typeof descriptor.value === 'function' ? descriptor.value.length : -1,
        ];
      } catch (error) {
        record[name] = ['error', String(error)];
      }
    }
    return record;
  };
  const names = Object.getOwnPropertyNames(window);
  const surface = { names: [], window: {}, entries: {} };
  for (const name of names) {
    surface.names.push(name);
    try {
      const descriptor = Object.getOwnPropertyDescriptor(window, name);
      surface.window[name] = [
        descriptor.get ? 1 : 0,
        descriptor.set ? 1 : 0,
        descriptor.enumerable ? 1 : 0,
        descriptor.configurable ? 1 : 0,
        'value' in descriptor && descriptor.writable === true ? 1 : 0,
        typeof descriptor.value === 'function' ? descriptor.value.length : -1,
      ];
    } catch (error) {
      surface.window[name] = ['error', String(error)];
    }
    let value;
    try {
      value = window[name];
    } catch (error) {
      continue;
    }
    if (value === null || (typeof value !== 'object' && typeof value !== 'function')) continue;
    const entry = { kind: typeof value, own: describe(value) };
    if (typeof value === 'function') {
      entry.name = value.name;
      entry.length = value.length;
      const proto = value.prototype;
      if (proto && typeof proto === 'object') {
        entry.proto = describe(proto);
        const parent = Object.getPrototypeOf(proto);
        entry.parent_proto = parent?.constructor?.name ?? null;
      }
      const parentCtor = Object.getPrototypeOf(value);
      entry.parent_ctor = parentCtor ? parentCtor.name : null;
    }
    surface.entries[name] = entry;
  }
  return JSON.stringify({
    url: location.href,
    user_agent: navigator.userAgent,
    webdriver: navigator.webdriver === true,
    surface,
  });
})()`;
