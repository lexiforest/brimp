Object.defineProperty(globalThis, '__brimpStorageHost', {
  configurable: true,
  value(request, callback) {
    try {
      return JSON.parse(__callHost('storageOperation', null, JSON.stringify(request), callback));
    } catch (error) {
      if (String(error).includes('SecurityError:'))
        throw new DOMException('Storage requires a non-opaque origin', 'SecurityError');
      throw error;
    }
  },
});
