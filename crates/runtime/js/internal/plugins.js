function installPlugins(descriptors, codec) {
  const profiles = globalThis.__brimpProfileRegistry;
  const native = globalThis.__brimpPluginHost;
  delete globalThis.__brimpPluginHost;
  const canvas = globalThis.__brimpCanvasBackends;
  delete globalThis.__brimpCanvasBackends;
  const PublicError = DOMException;
  let ready = false,
    error = null,
    closed = false,
    disposed = false,
    disposing = false;
  const subscriptions = descriptors.map(() => new Map());
  const disposers = [];
  const scopes = [];
  const { encode, ref } = codec;
  const functions = [];
  const decode = (wire, index) => codec.decode(wire, functions[index]);
  (async () => {
    for (let index = 0; index < descriptors.length; index++) {
      const descriptor = descriptors[index];
      const before = new Set(Object.getOwnPropertyNames(globalThis));
      try {
        if (closed) throw new Error('plugin realm was closed');
        const allowed = new Set(descriptor.handlers);
        functions[index] = (name) => {
          if (!allowed.has(name)) throw new TypeError(`unregistered host handler: ${name}`);
          return (...args) => host.call(name, ...args);
        };
        const host = Object.freeze({
          function: functions[index],
          ref,
          call(name, ...args) {
            try {
              codec.checkCall();
            } catch (error) {
              return Promise.reject(error);
            }
            return Promise.resolve()
              .then(() => {
                if (closed && !disposing) throw new Error('plugin realm was closed');
                if (!allowed.has(name)) throw new Error(`unregistered host handler: ${name}`);
                const wire = encode(args);
                try {
                  return native(index, name, JSON.stringify(wire.value));
                } catch (error) {
                  codec.release(wire);
                  throw error;
                }
              })
              .then(
                (wire) => decode(JSON.parse(wire), index),
                () => {
                  // Native code records the full failure for the host.
                  // Never pass adapter exceptions or host stacks to pages.
                  throw new PublicError('The operation failed.', 'OperationError');
                },
              );
          },
          subscribe(channel, listener) {
            if (closed) throw new Error('plugin realm was closed');
            if (typeof listener !== 'function')
              throw new TypeError('subscription listener must be callable');
            channel = String(channel);
            let listeners = subscriptions[index].get(channel);
            if (!listeners) subscriptions[index].set(channel, (listeners = new Set()));
            listeners.add(listener);
            return () => {
              listeners.delete(listener);
              if (!listeners.size) subscriptions[index].delete(channel);
            };
          },
        });
        // The final directive overrides file/source-map paths in trusted
        // installers. Keep their original locations only in host errors.
        const installer = (0, eval)(`(${descriptor.source}\n)\n//# sourceURL=web-api`);
        if (typeof installer !== 'function')
          throw new TypeError('plugin source must evaluate to an installer');
        const installed = await installer({
          global: globalThis,
          profile: profiles.forOwner(descriptor.name),
          host,
          canvas: Object.freeze({
            register(type, factory) {
              const dispose = canvas.register(type, factory);
              disposers.push(dispose);
              return dispose;
            },
          }),
        });
        const dispose =
          typeof installed === 'function' ? installed : installed?.dispose?.bind(installed);
        scopes.push({
          name: descriptor.name,
          globals: Object.getOwnPropertyNames(globalThis).filter((name) => !before.has(name)),
        });
        if (dispose) {
          if (closed) await dispose();
          else disposers.push(dispose);
        }
      } catch (cause) {
        const locations = Array.from(
          descriptor.source.matchAll(/\/\/[#@]\s*sourceURL=([^\r\n]+)/g),
          (match) => match[1],
        );
        throw new Error(
          `Plugin ${descriptor.name}${locations.length ? ` (${locations.join(', ')})` : ''}: ${String(cause)}\n${cause?.stack || ''}`,
        );
      }
    }
    Object.defineProperty(globalThis, '__brimpPluginScopes', {
      value: scopes,
      configurable: true,
    });
    ready = true;
  })().catch((cause) => {
    error = `${String(cause)}\n${cause?.stack || ''}`;
    ready = true;
  });
  return (serialized) => {
    const command = JSON.parse(serialized);
    if (command.op === 'dispose' && !closed) {
      closed = true;
      disposing = true;
      for (const channels of subscriptions) channels.clear();
      (async () => {
        for (const dispose of disposers.reverse()) {
          try {
            await dispose();
          } catch (cause) {
            error = `${String(cause)}\n${cause?.stack || ''}`;
          }
        }
        disposers.length = 0;
        disposing = false;
        disposed = true;
      })();
    } else if (command.op === 'notify' && !closed) {
      const { plugin, channel, value } = command.notification;
      for (const listener of subscriptions[plugin]?.get(channel) ?? []) {
        try {
          listener(decode(value, plugin));
        } catch (cause) {
          setTimeout(() => {
            throw cause;
          }, 0);
        }
      }
    }
    return JSON.stringify({ ready, error, disposed });
  };
}
