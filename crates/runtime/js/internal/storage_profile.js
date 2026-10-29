(() => {
  let storageProfileValues = null;
  Object.defineProperty(globalThis, '__brimpInstallStorageProfile', {
    value: (values) => {
      storageProfileValues = values;
    },
    configurable: true,
  });
  const profile = () => storageProfileValues;
  if (typeof StorageManager !== 'function') return;
  const estimate = StorageManager.prototype.estimate;
  const persisted = StorageManager.prototype.persisted;
  StorageManager.prototype.estimate = function () {
    const values = profile();
    if (values && values.quota !== null && values.quota !== undefined)
      return Promise.resolve({ quota: values.quota, usage: values.usage ?? 0 });
    return estimate.call(this);
  };
  StorageManager.prototype.persisted = function () {
    const values = profile();
    if (values && typeof values.persisted === 'boolean') return Promise.resolve(values.persisted);
    return persisted.call(this);
  };
})();
