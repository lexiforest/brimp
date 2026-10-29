function defineCoreProfile(profile) {
  const identity = profile.values(
    'identity',
    {
      user_agent:
        'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/27.0 Safari/605.1.15',
      app_code_name: 'Mozilla',
      app_name: 'Netscape',
      app_version:
        '5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/27.0 Safari/605.1.15',
      vendor: 'Apple Computer, Inc.',
      vendor_sub: '',
      product: 'Gecko',
      product_sub: '20030107',
      platform: 'MacIntel',
      languages: ['en-US'],
      locale: 'en-US',
      timezone: 'Asia/Taipei',
      doNotTrack: null,
      onLine: true,
      cookieEnabled: true,
      webdriver: false,
      oscpu: null,
      build_id: null,
    },
    {
      version: 1,
      fields: {
        doNotTrack: {
          type: 'string',
          nullable: true,
        },
        oscpu: {
          type: 'string',
          nullable: true,
        },
        build_id: {
          type: 'string',
          nullable: true,
        },
        languages: {
          nonempty: true,
          items: {
            type: 'string',
            nonempty: true,
          },
        },
      },
      capture: ({ read }) => ({
        user_agent: read(() => navigator.userAgent),
        app_code_name: read(() => navigator.appCodeName),
        app_name: read(() => navigator.appName),
        app_version: read(() => navigator.appVersion),
        vendor: read(() => navigator.vendor),
        vendor_sub: read(() => navigator.vendorSub),
        product: read(() => navigator.product),
        product_sub: read(() => navigator.productSub),
        platform: read(() => navigator.platform),
        languages: read(() => navigator.languages),
        locale: read(() => navigator.language),
        doNotTrack: read(() => navigator.doNotTrack),
        onLine: read(() => navigator.onLine),
        cookieEnabled: read(() => navigator.cookieEnabled),
        webdriver: read(() => navigator.webdriver),
        oscpu: read(() => navigator.oscpu),
        build_id: read(() => navigator.buildID),
        timezone: read(() => Intl.DateTimeFormat().resolvedOptions().timeZone),
      }),
    },
  );
  const device = profile.values(
    'device',
    {
      hardware_concurrency: 8,
      max_touch_points: 0,
    },
    {
      version: 1,
      capture: ({ read }) => ({
        hardware_concurrency: read(() => navigator.hardwareConcurrency),
        max_touch_points: read(() => navigator.maxTouchPoints),
      }),
    },
  );
  const screen = profile.values(
    'screen',
    {
      width: 1710,
      height: 1107,
      avail_width: 1710,
      avail_height: 1019,
      avail_left: 0,
      avail_top: 34,
      color_depth: 24,
      pixel_depth: 24,
      orientation: 'landscape-primary',
      device_pixel_ratio: 2.0,
    },
    {
      version: 1,
      capture: ({ read }) => ({
        width: read(() => screen.width),
        height: read(() => screen.height),
        avail_width: read(() => screen.availWidth),
        avail_height: read(() => screen.availHeight),
        avail_left: read(() => screen.availLeft),
        avail_top: read(() => screen.availTop),
        color_depth: read(() => screen.colorDepth),
        pixel_depth: read(() => screen.pixelDepth),
        orientation: read(() => screen.orientation?.type),
        device_pixel_ratio: read(() => devicePixelRatio),
      }),
    },
  );
  const window = profile.values(
    'window',
    {
      inner_width: 800,
      inner_height: 600,
      outer_width: 800,
      outer_height: 652,
      screen_x: 110,
      screen_y: 34,
    },
    {
      version: 1,
      capture: ({ read }) => ({
        inner_width: read(() => window.innerWidth),
        inner_height: read(() => window.innerHeight),
        outer_width: read(() => window.outerWidth),
        outer_height: read(() => window.outerHeight),
        screen_x: read(() => window.screenX),
        screen_y: read(() => window.screenY),
      }),
    },
  );
  const mediaQueries = profile.values(
    'mediaQueries',
    {
      prefers_color_scheme: 'light',
      prefers_reduced_motion: 'no-preference',
      prefers_contrast: 'no-preference',
      pointer: 'fine',
      hover: 'hover',
    },
    {
      version: 1,
      capture: ({ read }) => {
        const choice = (feature, values) =>
          read(() => values.find((v) => matchMedia(`(${feature}: ${v})`).matches));
        return {
          prefers_color_scheme: choice('prefers-color-scheme', ['light', 'dark']),
          prefers_reduced_motion: choice('prefers-reduced-motion', ['no-preference', 'reduce']),
          prefers_contrast: choice('prefers-contrast', ['no-preference', 'more', 'less', 'custom']),
          pointer: choice('pointer', ['fine', 'coarse', 'none']),
          hover: choice('hover', ['hover', 'none']),
        };
      },
    },
  );
  const navigatorPlugins = profile.values(
    'navigatorPlugins',
    {
      entries: [
        {
          name: 'PDF Viewer',
          filename: 'internal-pdf-viewer',
          description: 'Portable Document Format',
          mime_types: [
            {
              type: 'application/pdf',
              suffixes: 'pdf',
              description: 'Portable Document Format',
            },
            {
              type: 'text/pdf',
              suffixes: 'pdf',
              description: 'Portable Document Format',
            },
          ],
        },
        {
          name: 'WebKit built-in PDF',
          filename: 'internal-pdf-viewer',
          description: 'Portable Document Format',
          mime_types: [
            {
              type: 'application/pdf',
              suffixes: 'pdf',
              description: 'Portable Document Format',
            },
            {
              type: 'text/pdf',
              suffixes: 'pdf',
              description: 'Portable Document Format',
            },
          ],
        },
      ],
      pdfViewerEnabled: true,
    },
    {
      version: 1,
      capture: ({ read }) => ({
        entries: read(() =>
          Array.from(navigator.plugins, (p) => ({
            name: p.name,
            filename: p.filename,
            description: p.description,
            mime_types: Array.from(p, (m) => ({
              type: m.type,
              suffixes: m.suffixes,
              description: m.description,
            })),
          })),
        ),
        pdfViewerEnabled: read(() => navigator.pdfViewerEnabled),
      }),
    },
  );
  const fonts = profile.values(
    'fonts',
    {
      families: [
        'Helvetica',
        'Arial',
        'Times New Roman',
        'Courier New',
        'Verdana',
        'Georgia',
        'Trebuchet MS',
        'PingFang SC',
      ],
      metrics: {
        Helvetica: 0.5,
        Arial: 0.5,
        'Times New Roman': 0.5,
        'Courier New': 0.6,
        Verdana: 0.55,
        Georgia: 0.5,
        'Trebuchet MS': 0.52,
        'PingFang SC': 1.0,
      },
    },
    {
      version: 1,
      fields: {
        families: {
          items: {
            type: 'string',
          },
        },
        metrics: {
          type: 'object',
          additionalProperties: {
            type: 'number',
          },
        },
      },
      notes:
        'Candidate font detection and mean glyph advance measurements; not an exhaustive installed-font inventory or a shaping engine.',
      capture: ({ read, defaults }) => {
        const canvas = document.createElement('canvas'),
          ctx = canvas.getContext('2d');
        if (!ctx) throw new Error('Canvas 2D context unavailable');
        const text = 'mmmmmmmmmmlliWW0123456789';
        const width = (font) => {
          ctx.font = `100px ${font}`;
          return ctx.measureText(text).width;
        };
        const baselines = ['monospace', 'sans-serif', 'serif'];
        const baselineWidths = baselines.map(width);
        const families = defaults.families.filter(
          (f) =>
            baselines.includes(f) ||
            baselines.some((b, i) => width(`${JSON.stringify(f)},${b}`) !== baselineWidths[i]),
        );
        return {
          families: read(() => families),
          metrics: read(() =>
            Object.fromEntries(
              families.map((f) => [f, width(JSON.stringify(f)) / (100 * text.length)]),
            ),
          ),
        };
      },
    },
  );
  const timing = profile.values(
    'timing',
    {
      navigation_start_offset_ms: 180.0,
      dom_content_loaded_ms: 310.0,
      load_event_ms: 420.0,
    },
    {
      version: 1,
      capture: ({ read }) => {
        const timing = performance.timing;
        const offset = (name) => (timing ? timing[name] - timing.navigationStart : undefined);
        return {
          navigation_start_offset_ms: read(() => offset('fetchStart')),
          dom_content_loaded_ms: read(() => offset('domContentLoadedEventEnd')),
          load_event_ms: read(() => offset('loadEventEnd')),
        };
      },
    },
  );
  const network = profile.values(
    'network',
    {
      accept: 'text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8',
      accept_encoding: 'gzip, deflate, br',
      impersonation: 'safari270',
    },
    {
      version: 1,
      notes:
        'HTTP Accept/Accept-Encoding and TLS impersonation cannot be read reliably from page JavaScript; defaults are explicitly reported, not captured.',
    },
  );
  const storage = profile.values(
    'storage',
    {
      quota: null,
      usage: 0,
      persisted: false,
    },
    {
      version: 1,
      fields: {
        quota: {
          type: 'number',
          nullable: true,
          minimum: 0,
        },
        usage: {
          type: 'number',
          minimum: 0,
        },
        persisted: {
          type: 'boolean',
        },
      },
      notes:
        'StorageManager estimate/persisted answers captured from the claimed browser; usage is a point-in-time sample, not a live counter.',
      capture: async ({ read }) => ({
        quota: await read(async () => (await navigator.storage?.estimate())?.quota),
        usage: await read(async () => (await navigator.storage?.estimate())?.usage),
        persisted: await read(async () => navigator.storage?.persisted()),
      }),
    },
  );
  const css = profile.values(
    'css',
    {
      supports: [],
    },
    {
      version: 1,
      fields: {
        supports: {
          items: {
            type: 'string',
            nonempty: true,
          },
        },
      },
      notes:
        'Declarations the claimed browser answers true for CSS.supports. Checked before the native property registry so properties the DOM parser does not carry still answer like the browser.',
      capture: ({ read }) => ({
        supports: read(() => {
          const candidates = [
            ['aspect-ratio', '1 / 1'],
            ['backdrop-filter', 'blur(2px)'],
            ['container-type', 'inline-size'],
            ['container-name', 'sidebar'],
            ['display', 'grid'],
            ['display', 'flex'],
            ['gap', '1px'],
            ['inset', '0'],
            ['position', 'sticky'],
            ['object-fit', 'cover'],
            ['mix-blend-mode', 'multiply'],
            ['filter', 'blur(1px)'],
            ['transform', 'rotate(1deg)'],
            ['transition', 'all 1s'],
            ['animation', 'none 1s'],
            ['grid-template-columns', '1fr'],
            ['font-variation-settings', '"wght" 400'],
            ['text-wrap', 'balance'],
            ['accent-color', 'red'],
            ['color', 'color(display-p3 1 0 0)'],
            ['scrollbar-gutter', 'stable'],
            ['appearance', 'none'],
            ['background-color', 'oklch(0.5 0.1 200)'],
            ['border-radius', '1px'],
            ['box-shadow', '0 0 1px red'],
            ['user-select', 'none'],
            ['touch-action', 'none'],
            ['overscroll-behavior', 'contain'],
            ['content-visibility', 'auto'],
            ['contain-intrinsic-size', '1px'],
            ['view-transition-name', 'none'],
            ['scroll-timeline-name', 'none'],
            ['anchor-name', '--none'],
          ];
          const supported = [];
          for (const [property, value] of candidates) {
            try {
              if (CSS.supports(property, value)) supported.push(`${property}:${value}`);
            } catch (error) {}
          }
          return supported;
        }),
      }),
    },
  );
  return {
    identity,
    device,
    screen,
    window,
    media: mediaQueries,
    plugins: navigatorPlugins.entries,
    pdfViewerEnabled: navigatorPlugins.pdfViewerEnabled,
    fonts,
    timing,
    network,
    storage,
    css,
  };
}
