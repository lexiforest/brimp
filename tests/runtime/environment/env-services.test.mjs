import test from "node:test";
import assert from "node:assert/strict";
import {
  createSession,
  environmentPlugin,
  permissionsPlugin,
} from "../../../bindings/node/index.js";

test("matchMedia returns profile-evaluated MediaQueryList objects", async () => {
  const session = await createSession();
  try {
    const page = await session.newPage();
    assert.deepEqual(
      await page.evaluate(`(() => {
      const value = query => {
        const list = matchMedia(query);
        return [list.media, list.matches, list instanceof MediaQueryList,
          typeof list.addEventListener, typeof list.addListener, list.onchange];
      };
      return [
        value('(min-width: 700px)'),
        value('(max-width: 700px)'),
        value('(prefers-color-scheme: light)'),
        value('(prefers-color-scheme: dark)'),
        value('screen and (orientation: landscape)'),
        value('(hover)'),
        value('(color)'),
        value('not all'),
        value('(unknown-feature: value)'),
      ];
    })()`),
      [
        ["(min-width: 700px)", true, true, "function", "function", null],
        ["(max-width: 700px)", false, true, "function", "function", null],
        [
          "(prefers-color-scheme: light)",
          true,
          true,
          "function",
          "function",
          null,
        ],
        [
          "(prefers-color-scheme: dark)",
          false,
          true,
          "function",
          "function",
          null,
        ],
        [
          "screen and (orientation: landscape)",
          true,
          true,
          "function",
          "function",
          null,
        ],
        ["(hover)", true, true, "function", "function", null],
        ["(color)", true, true, "function", "function", null],
        ["not all", false, true, "function", "function", null],
        ["(unknown-feature: value)", false, true, "function", "function", null],
      ],
    );
  } finally {
    await session.close();
  }
});

test("document.fonts answers from profile families and retains added FontFace objects", async () => {
  const session = await createSession();
  try {
    const page = await session.newPage();
    assert.deepEqual(
      await page.evaluate(`(async () => {
      const fonts = document.fonts;
      const face = new FontFace('Local Face', 'local(Arial)');
      const initial = [fonts instanceof FontFaceSet, fonts.status, fonts.size,
        fonts.check('16px Helvetica'), fonts.check('16px Unknown Family'),
        fonts.check('italic bold 16px "Times New Roman", serif'),
        fonts.check('12px monospace'), fonts.check('')];
      const added = fonts.add(face);
      const after = [added === fonts, fonts.size, fonts.has(face), face.status,
        face.family, face.weight, face.display, face instanceof FontFace];
      const loaded = await face.load();
      const ready = await fonts.ready;
      const load = await fonts.load('16px Helvetica');
      const deleted = fonts.delete(face);
      return [initial, after, loaded === face, face.status, ready === fonts, load, deleted, fonts.size];
    })()`),
      [
        [true, "loaded", 0, true, false, true, true, false],
        [true, 1, true, "unloaded", "Local Face", "normal", "auto", true],
        true,
        "loaded",
        true,
        [],
        true,
        0,
      ],
    );
  } finally {
    await session.close();
  }
});

test("performance timing offsets come from the profile and memory stays absent", async () => {
  const session = await createSession();
  try {
    const page = await session.newPage();
    assert.deepEqual(
      await page.evaluate(`(() => {
      const timing = performance.timing;
      return [timing.navigationStart === performance.timeOrigin,
        timing.fetchStart - timing.navigationStart,
        timing.domContentLoadedEventStart - timing.navigationStart,
        timing.loadEventEnd - timing.navigationStart,
        typeof performance.memory];
    })()`),
      [true, 180, 310, 420, "undefined"],
    );
  } finally {
    await session.close();
  }
});

test("services plugin uses profile plugins, permissions and connection values", async () => {
  const session = await createSession({
    plugins: [environmentPlugin(), permissionsPlugin()],
    // navigator.connection is Chromium-only; Safari has no such surface.
    profile: {
      values: {
        identity: {
          user_agent:
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/156.0.0.0 Safari/537.36",
        },
      },
    },
  });
  try {
    const page = await session.newPage();
    assert.deepEqual(
      await page.evaluate(`(async () => ({
      plugins: [navigator.plugins.length, [...navigator.plugins].map(plugin => plugin.name)],
      mimeTypes: [navigator.mimeTypes.length, [...navigator.mimeTypes].map(mime => mime.type)],
      notification: Notification.permission,
      permission: (await navigator.permissions.query({name: 'geolocation'})).state,
      connection: [navigator.connection.type, navigator.connection.rtt,
        navigator.connection.downlink, navigator.connection.effectiveType],
      deviceMemory: 'deviceMemory' in navigator,
      internal: Object.getOwnPropertyNames(globalThis).filter(name => name.startsWith('__brimp')),
    }))()`),
      {
        plugins: [2, ["PDF Viewer", "WebKit built-in PDF"]],
        mimeTypes: [2, ["application/pdf", "text/pdf"]],
        notification: "default",
        permission: "prompt",
        connection: ["wifi", 50, 10, "4g"],
        deviceMemory: true,
        internal: [],
      },
    );
  } finally {
    await session.close();
  }
});

test("a supplied profile changes media, font and timing answers", async () => {
  const custom = { values: { mediaQueries: {}, fonts: {}, timing: {} } };
  custom.values.mediaQueries.prefers_color_scheme = "dark";
  custom.values.mediaQueries.prefers_reduced_motion = "reduce";
  custom.values.fonts.families = ["Arial"];
  custom.values.fonts.metrics = { Arial: 0.5 };
  custom.values.timing.navigation_start_offset_ms = 10;
  custom.values.timing.dom_content_loaded_ms = 20;
  custom.values.timing.load_event_ms = 30;
  const session = await createSession({ profile: custom });
  try {
    const page = await session.newPage();
    assert.deepEqual(
      await page.evaluate(`(() => {
      const timing = performance.timing;
      return [
        matchMedia('(prefers-color-scheme: dark)').matches,
        matchMedia('(prefers-reduced-motion: reduce)').matches,
        document.fonts.check('16px Helvetica'),
        document.fonts.check('16px Arial'),
        timing.domContentLoadedEventEnd - timing.navigationStart,
        timing.loadEventStart - timing.navigationStart,
      ];
    })()`),
      [true, true, false, true, 20, 30],
    );
  } finally {
    await session.close();
  }
});
