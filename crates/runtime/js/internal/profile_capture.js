async function captureBrowserProfile(contract, probes, createRegistry, options = {}) {
  const results = Object.create(null);
  const values = Object.create(null);
  const errorResult = (error) => ({
    status: ['NotAllowedError', 'SecurityError'].includes(error?.name)
      ? 'permission-required'
      : 'failed',
    reason: String(error?.message ?? error),
  });
  // JSON serialization must not silently turn an unsupported observation into null.
  function copy(value) {
    return JSON.parse(
      JSON.stringify(value, (_, item) => {
        if (
          item === undefined ||
          typeof item === 'function' ||
          typeof item === 'symbol' ||
          typeof item === 'bigint' ||
          (typeof item === 'number' && !Number.isFinite(item))
        ) {
          throw new TypeError('Measurement is not a JSON value');
        }
        return item;
      }),
    );
  }
  function read(getter) {
    const measured = (value) =>
      value === undefined
        ? { status: 'absent', reason: 'The browser does not expose this value' }
        : { status: 'captured', value: copy(value) };
    try {
      const value = getter();
      return value && typeof value.then === 'function'
        ? Promise.resolve(value).then(measured).catch(errorResult)
        : measured(value);
    } catch (error) {
      return errorResult(error);
    }
  }
  async function readRecord(getters) {
    const entries = await Promise.all(
      Object.entries(getters).map(async ([key, getter]) => [key, await read(getter)]),
    );
    const value = Object.fromEntries(
      entries
        .filter(([, result]) => result.status === 'captured')
        .map(([key, result]) => [key, result.value]),
    );
    return {
      status: Object.keys(value).length ? 'captured' : 'unavailable',
      value,
      reason: 'No record entries could be measured',
      children: Object.fromEntries(entries.map(([key, { value, ...result }]) => [key, result])),
    };
  }
  async function bounded(value) {
    let timer;
    try {
      return await Promise.race([
        Promise.resolve(value),
        new Promise((_, reject) => {
          timer = setTimeout(
            () => reject(new Error('Browser measurement timed out after 10 seconds')),
            10000,
          );
        }),
      ]);
    } finally {
      clearTimeout(timer);
    }
  }
  for (const entry of contract.interfaces) {
    const defaults = Object.fromEntries(
      Object.entries(entry.fields).map(([key, field]) => [key, field.default]),
    );
    let observations = {};
    let failure;
    try {
      if (probes[entry.scope])
        observations = await bounded(
          probes[entry.scope]({ read, readRecord, defaults: copy(defaults) }),
        );
      if (!observations || typeof observations !== 'object')
        throw new TypeError('Probe must return field results');
      for (const key of Object.keys(observations)) {
        if (!Object.hasOwn(entry.fields, key))
          throw new TypeError(`Probe returned unknown field ${key}`);
      }
    } catch (error) {
      failure = errorResult(error);
    }
    const supplied = Object.create(null);
    for (const key of Object.keys(entry.fields)) {
      let result;
      try {
        result = failure ??
          (await bounded(observations[key])) ?? {
            status: 'unavailable',
            reason: 'No browser measurement is defined for this setting',
          };
        if (
          !['captured', 'absent', 'permission-required', 'failed', 'unavailable'].includes(
            result.status,
          )
        ) {
          throw new TypeError('Invalid probe result status');
        }
        for (const [child, childResult] of Object.entries(result.children ?? {})) {
          const path = `${entry.scope}.${key}.${child}`;
          results[path] = copy(childResult);
          options.onProgress?.({ path, ...childResult });
        }
        if (result.status === 'captured') {
          const value = copy(result.value);
          const registry = createRegistry({ values: { [entry.scope]: { [key]: value } } });
          registry.forOwner(entry.owner).values(entry.scope, defaults, { fields: entry.fields });
          supplied[key] = value;
          result = { status: 'captured' };
        } else {
          result = { status: result.status, reason: String(result.reason ?? result.status) };
        }
      } catch (error) {
        result = errorResult(error);
      }
      const path = `${entry.scope}.${key}`;
      results[path] = result;
      options.onProgress?.({ path, ...result });
    }
    if (Object.keys(supplied).length) values[entry.scope] = supplied;
  }
  const profile = {
    format: 'brimp-profile',
    version: 1,
    values,
    capture: {
      toolVersion: options.toolVersion,
      capturedAt: new Date().toISOString(),
      userAgent: globalThis.navigator?.userAgent ?? null,
      selectedPlugins: options.selectedPlugins ?? [],
      interfaces: contract.interfaces.map(({ scope, owner, version }) => ({
        scope,
        owner,
        version,
      })),
      results,
    },
  };
  const registry = createRegistry(profile);
  for (const entry of contract.interfaces) {
    registry
      .forOwner(entry.owner)
      .values(
        entry.scope,
        Object.fromEntries(
          Object.entries(entry.fields).map(([key, field]) => [key, field.default]),
        ),
        { fields: entry.fields, version: entry.version },
      );
  }
  registry.validate();
  profile.capture.defaultsUsed = registry.report().defaultsUsed;
  return profile;
}
