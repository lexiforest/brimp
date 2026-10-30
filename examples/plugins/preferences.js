// A deliberately asynchronous API. The host owns the lifetime of its data.
({ global, host }) => {
  global.preferences = Object.freeze({
    get: (key) => host.call('get', String(key)),
    set: (key, value) => host.call('set', String(key), value),
  });
  return () => {
    delete global.preferences;
  };
};
