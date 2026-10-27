function createProfileRegistry(document = {}) {
  const stringifyFunction = Function.prototype.toString;
  const definitions = new Map();
  const input = document.values ?? {};
  const resolved = Object.create(null);
  const defaultsUsed = [];
  const own = (object, key) => Object.prototype.hasOwnProperty.call(object, key);
  const object = (value) => value !== null && typeof value === 'object' && !Array.isArray(value);
  const clone = (value) => JSON.parse(JSON.stringify(value));
  const fail = (path, message) => {
    throw new TypeError(`Profile ${path}: ${message}`);
  };
  if (!object(document) || !object(input)) fail('values', 'expected an object');
  for (const key of Object.keys(document)) {
    if (!['format', 'version', 'values', 'capture'].includes(key))
      fail(key, 'unknown document field');
  }
  if (document.format !== undefined && document.format !== 'brimp-profile')
    fail('format', 'expected brimp-profile');
  if (document.version !== undefined && document.version !== 1) fail('version', 'expected 1');

  function infer(value, hint = {}) {
    const type =
      hint.type ?? (value === null ? 'json' : Array.isArray(value) ? 'array' : typeof value);
    const schema = { ...hint, type };
    if (type === 'object' && !hint.additionalProperties) {
      schema.properties = Object.fromEntries(
        Object.entries(value ?? {}).map(([key, item]) => [
          key,
          infer(item, hint.properties?.[key]),
        ]),
      );
    }
    if (type === 'array' && !schema.items && value?.length) schema.items = infer(value[0]);
    return schema;
  }
  function check(value, schema, path) {
    if (value === null && (schema.nullable || schema.type === 'json')) return;
    switch (schema.type) {
      case 'json':
        break;
      case 'array':
        if (!Array.isArray(value)) fail(path, 'expected an array');
        if (schema.items) value.forEach((item, i) => check(item, schema.items, `${path}[${i}]`));
        break;
      case 'object':
        if (!object(value)) fail(path, 'expected an object');
        for (const [key, item] of Object.entries(value)) {
          if (schema.properties && own(schema.properties, key))
            check(item, schema.properties[key], `${path}.${key}`);
          else if (schema.additionalProperties && typeof schema.additionalProperties === 'object')
            check(item, schema.additionalProperties, `${path}.${key}`);
          else if (!schema.additionalProperties)
            fail(
              `${path}.${key}`,
              `unknown key; available: ${Object.keys(schema.properties ?? {}).join(', ')}`,
            );
        }
        break;
      default:
        if (typeof value !== schema.type) fail(path, `expected ${schema.type}`);
    }
    if (typeof value === 'number' && !Number.isFinite(value))
      fail(path, 'expected a finite number');
    if (schema.minimum !== undefined && value < schema.minimum)
      fail(path, `must be at least ${schema.minimum}`);
    if (schema.maximum !== undefined && value > schema.maximum)
      fail(path, `must be at most ${schema.maximum}`);
    if (schema.integer && !Number.isInteger(value)) fail(path, 'expected an integer');
    if (schema.enum && !schema.enum.includes(value))
      fail(path, `expected one of ${schema.enum.join(', ')}`);
    if (schema.nonempty && !value.length) fail(path, 'must not be empty');
  }
  function merge(defaults, supplied, path) {
    if (!object(defaults) || !object(supplied)) return clone(supplied);
    return Object.fromEntries(
      [...new Set([...Object.keys(defaults), ...Object.keys(supplied)])].map((key) => [
        key,
        !own(supplied, key)
          ? (defaultsUsed.push(`${path}.${key}`), clone(defaults[key]))
          : own(defaults, key)
            ? merge(defaults[key], supplied[key], `${path}.${key}`)
            : clone(supplied[key]),
      ]),
    );
  }
  function register(owner, scope, defaults, options = {}) {
    if (
      !/^[a-zA-Z][a-zA-Z0-9_-]*$/.test(scope) ||
      ['__proto__', 'constructor', 'prototype'].includes(scope)
    )
      fail(scope, 'invalid scope name');
    if (definitions.has(scope)) fail(scope, 'scope already registered');
    if (!object(defaults)) fail(scope, 'defaults must be an object');
    if (!Number.isSafeInteger(options.version ?? 1) || (options.version ?? 1) < 1)
      fail(scope, 'interface version must be a positive integer');
    const fields = Object.fromEntries(
      Object.entries(defaults).map(([key, value]) => [
        key,
        { default: clone(value), ...infer(value, options.fields?.[key]) },
      ]),
    );
    const supplied = input[scope] ?? {};
    if (!object(supplied)) fail(scope, 'expected an object');
    for (const key of Object.keys(supplied)) {
      if (!own(fields, key))
        fail(`${scope}.${key}`, `unknown key; available: ${Object.keys(fields).join(', ')}`);
      check(supplied[key], fields[key], `${scope}.${key}`);
    }
    for (const [key, field] of Object.entries(fields)) {
      check(defaults[key], field, `${scope}.${key} (default)`);
    }
    resolved[scope] = merge(defaults, supplied, scope);
    definitions.set(scope, {
      scope,
      owner,
      version: options.version ?? 1,
      fields,
      probe: options.capture ? stringifyFunction.call(options.capture) : null,
      notes: options.notes ?? null,
    });
    return resolved[scope];
  }
  return Object.freeze({
    forOwner(owner) {
      return Object.freeze({
        values: (scope, defaults, options) => register(owner, scope, defaults, options),
      });
    },
    resolved() {
      return clone(resolved);
    },
    contract() {
      return {
        format: 'brimp-profile-contract',
        version: 1,
        interfaces: [...definitions.values()],
      };
    },
    report() {
      return { defaultsUsed: defaultsUsed.slice(), capture: document.capture ?? null };
    },
    validate() {
      for (const scope of Object.keys(input)) {
        if (!definitions.has(scope))
          fail(
            scope,
            `unknown or unloaded interface; available: ${[...definitions.keys()].join(', ')}`,
          );
      }
      if (document.capture?.interfaces !== undefined) {
        if (!Array.isArray(document.capture.interfaces))
          fail('capture.interfaces', 'expected an array');
        const seen = new Set();
        for (const captured of document.capture.interfaces) {
          if (!object(captured) || typeof captured.scope !== 'string' || seen.has(captured.scope))
            fail('capture.interfaces', 'expected unique interface records');
          seen.add(captured.scope);
          const current = definitions.get(captured.scope);
          if (!current) fail(captured.scope, 'captured interface is not loaded');
          if (captured.owner !== current.owner || captured.version !== current.version)
            fail(
              captured.scope,
              `capture requires ${captured.owner} version ${captured.version}; loaded ${current.owner} version ${current.version}`,
            );
        }
      }
      return true;
    },
  });
}
