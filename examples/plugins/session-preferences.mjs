import {createSession, Plugin} from '../../bindings/node/index.js';

const preferences = Plugin.fromFile(new URL('./preferences.js', import.meta.url), {
  name: 'preferences',
  setup() {
    // This factory runs once per session, even when the Plugin is reused.
    const origins = new Map();
    function partition(context) {
      if (context.origin === null) throw new Error('Preferences require a non-opaque origin');
      if (!origins.has(context.origin)) origins.set(context.origin, new Map());
      return origins.get(context.origin);
    }
    return {
      handlers: {
        get: (context, key) => partition(context).get(key) ?? null,
        set: (context, key, value) => { partition(context).set(key, value); return null; },
      },
      dispose: () => origins.clear(),
    };
  },
});
const session = await createSession({plugins: [preferences]});
try {
  const page = await session.get(process.argv[2] ?? 'https://example.com');
  await page.evaluate('preferences.set("theme", "dark")');
  await page.get(process.argv[2] ?? 'https://example.com');
  console.log(await page.evaluate('preferences.get("theme")'));
} finally { await session.close(); }
