function installPermissions({ global, profile: settings }, options = {}) {
  const permissionProfile = settings.values(
    'permissions',
    {
      states: {
        'clipboard-read': 'prompt',
        geolocation: 'prompt',
        notifications: 'default',
        'persistent-storage': 'prompt',
      },
      notification_permission: 'default',
      permissions_enabled: true,
    },
    {
      version: 1,
      fields: {
        states: {
          type: 'object',
          additionalProperties: {
            type: 'string',
          },
        },
      },
      notes:
        'permissions_enabled is a runtime exposure choice; capture never disables its compatibility interface.',
      capture: async ({ read, readRecord, defaults }) => {
        return {
          states: await readRecord(
            Object.fromEntries(
              Object.keys(defaults.states).map((name) => [
                name,
                async () =>
                  navigator.permissions
                    ? (await navigator.permissions.query({ name })).state
                    : undefined,
              ]),
            ),
          ),
          notification_permission: read(() => globalThis.Notification?.permission),
        };
      },
    },
  );
  const navigatorValues = {
    notification_permission: permissionProfile.notification_permission,
    permissions_enabled: permissionProfile.permissions_enabled,
    ...options,
  };
  const constructToken = Symbol('browser interface construction');
  const defineGlobalConstructor = (name, constructor) => {
    if (!Object.hasOwn(constructor.prototype, Symbol.toStringTag)) {
      Object.defineProperty(constructor.prototype, Symbol.toStringTag, {
        value: name,
        configurable: true,
      });
    }
    Object.defineProperty(global, name, {
      value: constructor,
      writable: true,
      configurable: true,
    });
  };
  const defineNavigatorValue = (name, value) => {
    const getter = function () {
      return value;
    };
    Object.defineProperty(getter, 'name', { value: `get ${name}`, configurable: true });
    Object.defineProperty(Navigator.prototype, name, {
      get: getter,
      enumerable: true,
      configurable: true,
    });
  };
  {
    function Notification(title, options = {}) {
      this.title = String(title);
      this.body = options.body === undefined ? '' : String(options.body);
      this.close = () => undefined;
    }
    Object.defineProperty(Notification, 'permission', {
      get: () => navigatorValues.notification_permission,
    });
    Notification.requestPermission = (callback) => {
      const permission = navigatorValues.notification_permission;
      if (typeof callback === 'function') callback(permission);
      return Promise.resolve(permission);
    };
    defineGlobalConstructor('Notification', Notification);
  }

  class Permissions {
    constructor(token) {
      if (token !== constructToken) throw new TypeError('Illegal constructor');
    }
    query(descriptor) {
      const profileState = descriptor ? permissionProfile.states[descriptor.name] : undefined;
      const state =
        descriptor && descriptor.name === 'notifications'
          ? navigatorValues.notification_permission === 'default'
            ? 'prompt'
            : navigatorValues.notification_permission
          : (profileState ?? 'prompt');
      return Promise.resolve(
        Object.freeze({
          state,
          onchange: null,
          addEventListener() {},
          removeEventListener() {},
        }),
      );
    }
  }
  defineGlobalConstructor('Permissions', Permissions);
  if (navigatorValues.permissions_enabled) {
    defineNavigatorValue('permissions', new Permissions(constructToken));
  } else {
    delete Navigator.prototype.permissions;
  }

  class CredentialsContainer {
    constructor(token) {
      if (token !== constructToken) throw new TypeError('Illegal constructor');
    }
    create() {}
    get() {}
    preventSilentAccess() {}
    store() {}
  }
  defineGlobalConstructor('CredentialsContainer', CredentialsContainer);
  defineNavigatorValue('credentials', new CredentialsContainer(constructToken));
}
