function installCanvas({ global, canvas, profile: settings }, options = {}) {
  const profile = globalThis.__brimpProfile;
  const canvasProfile = settings.values(
    'canvas',
    {
      rgba: [0, 0, 0, 0],
      fallbackAdvance: 0.5,
    },
    {
      version: 1,
      capture: ({ read }) => {
        const canvas = document.createElement('canvas'),
          ctx = canvas.getContext('2d');
        if (!ctx) throw new Error('Canvas 2D context unavailable');
        ctx.font = '100px sans-serif';
        const text = 'mmmmmmmmmmlliWW0123456789';
        return {
          rgba: read(() => Array.from(ctx.getImageData(0, 0, 1, 1).data)),
          fallbackAdvance: read(() => ctx.measureText(text).width / (100 * text.length)),
        };
      },
    },
  );

  const rgba = options.rgba ?? canvasProfile.rgba;
  if (
    !Array.isArray(rgba) ||
    rgba.length !== 4 ||
    !rgba.every((v) => Number.isInteger(v) && v >= 0 && v <= 255)
  )
    throw new TypeError('rgba must contain four bytes');
  const fallbackAdvance = options.characterWidth ?? canvasProfile.fallbackAdvance;
  if (!Number.isFinite(fallbackAdvance) || fallbackAdvance < 0)
    throw new TypeError('characterWidth must be nonnegative');
  const alpha = new WeakMap();
  let nextResource = 1;
  const surfaces = new WeakMap();

  function surface(canvas, width, height) {
    let value = surfaces.get(canvas);
    if (!value || value.width !== width || value.height !== height) {
      value = { width, height, data: new Uint8ClampedArray(width * height * 4) };
      for (let index = 0; index < value.data.length; index += 4) {
        value.data[index] = rgba[0];
        value.data[index + 1] = rgba[1];
        value.data[index + 2] = rgba[2];
        value.data[index + 3] = rgba[3];
      }
      surfaces.set(canvas, value);
    }
    return value;
  }

  function resetSurface(canvas, width, height) {
    surfaces.delete(canvas);
    surface(canvas, width, height);
  }

  function paintRect(
    canvas,
    width,
    height,
    x,
    y,
    rectWidth,
    rectHeight,
    color,
    transform,
    composite,
  ) {
    if (
      !color ||
      !Number.isFinite(x) ||
      !Number.isFinite(y) ||
      !Number.isFinite(rectWidth) ||
      !Number.isFinite(rectHeight) ||
      rectWidth === 0 ||
      rectHeight === 0
    )
      return;
    if (rectWidth < 0) {
      x += rectWidth;
      rectWidth = -rectWidth;
    }
    if (rectHeight < 0) {
      y += rectHeight;
      rectHeight = -rectHeight;
    }
    const [a, b, c, d, e, f] = transform;
    const determinant = a * d - b * c;
    if (!determinant) return;
    const inverse = [
      d / determinant,
      -b / determinant,
      -c / determinant,
      a / determinant,
      (c * f - d * e) / determinant,
      (b * e - a * f) / determinant,
    ];
    const corners = [
      [x, y],
      [x + rectWidth, y],
      [x, y + rectHeight],
      [x + rectWidth, y + rectHeight],
    ].map(([px, py]) => [a * px + c * py + e, b * px + d * py + f]);
    const left = Math.max(0, Math.floor(Math.min(...corners.map((point) => point[0]))));
    const top = Math.max(0, Math.floor(Math.min(...corners.map((point) => point[1]))));
    const right = Math.min(width, Math.ceil(Math.max(...corners.map((point) => point[0]))));
    const bottom = Math.min(height, Math.ceil(Math.max(...corners.map((point) => point[1]))));
    const target = surface(canvas, width, height);
    const copy = composite === 'copy';
    const source = color;
    const sourceAlpha = Math.max(0, Math.min(1, source[3]));
    for (let py = top; py < bottom; py++) {
      for (let px = left; px < right; px++) {
        const localX = inverse[0] * (px + 0.5) + inverse[2] * (py + 0.5) + inverse[4];
        const localY = inverse[1] * (px + 0.5) + inverse[3] * (py + 0.5) + inverse[5];
        if (localX < x || localX >= x + rectWidth || localY < y || localY >= y + rectHeight)
          continue;
        const index = (py * width + px) * 4;
        if (copy || sourceAlpha >= 1) {
          target.data[index] = source[0] * 255;
          target.data[index + 1] = source[1] * 255;
          target.data[index + 2] = source[2] * 255;
          target.data[index + 3] = sourceAlpha * 255;
          continue;
        }
        const destinationAlpha = target.data[index + 3] / 255;
        const outAlpha = sourceAlpha + destinationAlpha * (1 - sourceAlpha);
        if (outAlpha <= 0) {
          target.data[index] =
            target.data[index + 1] =
            target.data[index + 2] =
            target.data[index + 3] =
              0;
          continue;
        }
        for (let channel = 0; channel < 3; channel++) {
          target.data[index + channel] =
            (source[channel] * 255 * sourceAlpha +
              target.data[index + channel] * destinationAlpha * (1 - sourceAlpha)) /
            outAlpha;
        }
        target.data[index + 3] = outAlpha * 255;
      }
    }
    if (alpha.get(canvas) === false) {
      for (let py = top; py < bottom; py++) {
        for (let px = left; px < right; px++) target.data[(py * width + px) * 4 + 3] = 255;
      }
    }
  }

  function clearPixels(canvas, width, height, x, y, rectWidth, rectHeight, transform) {
    if ([x, y, rectWidth, rectHeight].some((value) => !Number.isFinite(value))) return;
    if (rectWidth < 0) {
      x += rectWidth;
      rectWidth = -rectWidth;
    }
    if (rectHeight < 0) {
      y += rectHeight;
      rectHeight = -rectHeight;
    }
    const [a, b, c, d, e, f] = transform;
    const determinant = a * d - b * c;
    if (!determinant) return;
    const inverse = [
      d / determinant,
      -b / determinant,
      -c / determinant,
      a / determinant,
      (c * f - d * e) / determinant,
      (b * e - a * f) / determinant,
    ];
    const corners = [
      [x, y],
      [x + rectWidth, y],
      [x, y + rectHeight],
      [x + rectWidth, y + rectHeight],
    ].map(([px, py]) => [a * px + c * py + e, b * px + d * py + f]);
    const left = Math.max(0, Math.floor(Math.min(...corners.map((point) => point[0]))));
    const top = Math.max(0, Math.floor(Math.min(...corners.map((point) => point[1]))));
    const right = Math.min(width, Math.ceil(Math.max(...corners.map((point) => point[0]))));
    const bottom = Math.min(height, Math.ceil(Math.max(...corners.map((point) => point[1]))));
    const target = surface(canvas, width, height);
    const clearAlpha = alpha.get(canvas) === false ? 255 : 0;
    for (let py = top; py < bottom; py++) {
      for (let px = left; px < right; px++) {
        const localX = inverse[0] * (px + 0.5) + inverse[2] * (py + 0.5) + inverse[4];
        const localY = inverse[1] * (px + 0.5) + inverse[3] * (py + 0.5) + inverse[5];
        if (localX < x || localX >= x + rectWidth || localY < y || localY >= y + rectHeight)
          continue;
        const index = (py * width + px) * 4;
        target.data[index] = target.data[index + 1] = target.data[index + 2] = 0;
        target.data[index + 3] = clearAlpha;
      }
    }
  }

  function putPixels(canvas, width, height, x, y, imageWidth, imageHeight, source, alphaChannel) {
    const target = surface(canvas, width, height);
    for (let row = 0; row < imageHeight; row++) {
      const destinationRow = y + row;
      if (destinationRow < 0 || destinationRow >= height) continue;
      for (let column = 0; column < imageWidth; column++) {
        const destinationColumn = x + column;
        if (destinationColumn < 0 || destinationColumn >= width) continue;
        const from = (row * imageWidth + column) * 4;
        const to = (destinationRow * width + destinationColumn) * 4;
        target.data[to] = source[from];
        target.data[to + 1] = source[from + 1];
        target.data[to + 2] = source[from + 2];
        target.data[to + 3] = alphaChannel === false ? 255 : source[from + 3];
      }
    }
  }

  const crcTable = (() => {
    const table = new Uint32Array(256);
    for (let index = 0; index < 256; index++) {
      let value = index;
      for (let bit = 0; bit < 8; bit++)
        value = value & 1 ? 0xedb88320 ^ (value >>> 1) : value >>> 1;
      table[index] = value >>> 0;
    }
    return table;
  })();
  function crc32(bytes) {
    let crc = 0xffffffff;
    for (const byte of bytes) crc = crcTable[(crc ^ byte) & 0xff] ^ (crc >>> 8);
    return (crc ^ 0xffffffff) >>> 0;
  }
  function adler32(bytes) {
    let a = 1,
      b = 0;
    for (const byte of bytes) {
      a = (a + byte) % 65521;
      b = (b + a) % 65521;
    }
    return ((b << 16) | a) >>> 0;
  }
  function deflateStored(bytes) {
    const output = [0x78, 0x01];
    if (bytes.length === 0) output.push(1, 0, 0, 0xff, 0xff);
    for (let offset = 0; offset < bytes.length; offset += 65535) {
      const length = Math.min(65535, bytes.length - offset);
      const final = offset + length >= bytes.length ? 1 : 0;
      output.push(
        final,
        length & 0xff,
        (length >>> 8) & 0xff,
        ~length & 0xff,
        (~length >>> 8) & 0xff,
      );
      for (let index = 0; index < length; index++) output.push(bytes[offset + index]);
    }
    const check = adler32(bytes);
    output.push((check >>> 24) & 0xff, (check >>> 16) & 0xff, (check >>> 8) & 0xff, check & 0xff);
    return Uint8Array.from(output);
  }
  function pngChunk(type, data) {
    const typeBytes = Uint8Array.from(type, (character) => character.charCodeAt(0));
    const body = new Uint8Array(typeBytes.length + data.length);
    body.set(typeBytes);
    body.set(data, typeBytes.length);
    const crc = crc32(body);
    const chunk = new Uint8Array(4 + body.length + 4);
    new DataView(chunk.buffer).setUint32(0, data.length);
    chunk.set(body, 4);
    new DataView(chunk.buffer).setUint32(4 + body.length, crc);
    return chunk;
  }
  function pngDataURL(width, height, data, alphaChannel) {
    const rowLength = width * 4;
    const raw = new Uint8Array(height * (rowLength + 1));
    for (let row = 0; row < height; row++) {
      const start = row * (rowLength + 1);
      raw[start] = 0;
      raw.set(data.subarray(row * rowLength, (row + 1) * rowLength), start + 1);
      if (alphaChannel === false) {
        for (let column = 0; column < width; column++) raw[start + 1 + column * 4 + 3] = 255;
      }
    }
    const header = new Uint8Array(13);
    const view = new DataView(header.buffer);
    view.setUint32(0, width);
    view.setUint32(4, height);
    header[8] = 8;
    header[9] = 6;
    header[10] = 0;
    header[11] = 0;
    header[12] = 0;
    const chunks = [
      Uint8Array.of(0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a),
      pngChunk('IHDR', header),
      pngChunk('IDAT', deflateStored(raw)),
      pngChunk('IEND', new Uint8Array(0)),
    ];
    let length = 0;
    for (const chunk of chunks) length += chunk.length;
    const png = new Uint8Array(length);
    let offset = 0;
    for (const chunk of chunks) {
      png.set(chunk, offset);
      offset += chunk.length;
    }
    let binary = '';
    for (let index = 0; index < png.length; index += 0x8000) {
      binary += String.fromCharCode.apply(null, png.subarray(index, index + 0x8000));
    }
    return 'data:image/png;base64,' + btoa(binary);
  }

  function measureTextWidth(text, size, family) {
    const normalized = String(family).toLowerCase();
    const metrics = profile?.fonts?.metrics ?? {};
    let advance = options.characterWidth;
    for (const [name, value] of Object.entries(metrics)) {
      if (name.toLowerCase() === normalized) {
        advance = value;
        break;
      }
    }
    if (advance === undefined) advance = normalized === 'monospace' ? 0.6 : fallbackAdvance;
    let width = 0;
    for (const character of Array.from(String(text)))
      width += character.codePointAt(0) > 0xff ? 1 : advance;
    return width * size;
  }

  function call(operation, receiver, ...args) {
    switch (operation) {
      case 'canvasOriginClean':
        return true;
      case 'canvas2dAcquire':
        alpha.set(receiver, args[2]);
        surface(receiver, args[0], args[1]);
        return true;
      case 'canvasReset':
        resetSurface(receiver, args[0], args[1]);
        return undefined;
      case 'canvas2dCreateGradient':
      case 'canvas2dCreatePath':
      case 'canvas2dCreatePattern':
      case 'canvas2dCreateImagePattern':
      case 'canvas2dCreateImageBitmapPattern':
        return nextResource++;
      case 'canvas2dIsPointInPath':
      case 'canvas2dIsPointInStroke':
        return false;
      case 'canvas2dMeasureText': {
        const width = measureTextWidth(args[0], args[1], args[2]);
        return JSON.stringify([
          width,
          0,
          width,
          args[1] * 0.8,
          args[1] * 0.2,
          args[1] * 0.8,
          args[1] * 0.2,
        ]);
      }
      case 'canvas2dFillRect': {
        const style = JSON.parse(args[6] || '{}');
        if (style.color)
          paintRect(
            receiver,
            args[0],
            args[1],
            args[2],
            args[3],
            args[4],
            args[5],
            style.color,
            args.slice(7, 13),
            args[14],
          );
        return undefined;
      }
      case 'canvas2dStrokeRect': {
        const style = JSON.parse(args[6] || '{}');
        if (!style.color) return undefined;
        const stroke = JSON.parse(args[13] || '{}');
        const lineWidth = Math.max(1, Number(stroke.width) || 1);
        const [x, y, width, height] = args.slice(2, 6);
        const transform = args.slice(7, 13);
        const composite = args[15];
        paintRect(
          receiver,
          args[0],
          args[1],
          x,
          y,
          width,
          lineWidth,
          style.color,
          transform,
          composite,
        );
        paintRect(
          receiver,
          args[0],
          args[1],
          x,
          y + height - lineWidth,
          width,
          lineWidth,
          style.color,
          transform,
          composite,
        );
        paintRect(
          receiver,
          args[0],
          args[1],
          x,
          y,
          lineWidth,
          height,
          style.color,
          transform,
          composite,
        );
        paintRect(
          receiver,
          args[0],
          args[1],
          x + width - lineWidth,
          y,
          lineWidth,
          height,
          style.color,
          transform,
          composite,
        );
        return undefined;
      }
      case 'canvas2dClearRect':
        clearPixels(
          receiver,
          args[0],
          args[1],
          args[2],
          args[3],
          args[4],
          args[5],
          args.slice(6, 12),
        );
        return undefined;
      case 'canvas2dPutImageData': {
        const source = args[6];
        putPixels(
          receiver,
          args[0],
          args[1],
          args[2],
          args[3],
          args[4],
          args[5],
          source,
          alpha.get(receiver) === false,
        );
        return undefined;
      }
      case 'canvas2dGetImageData': {
        const [canvasWidth, canvasHeight, x, y, width, height, , format] = args;
        if (width * height > 16 * 1024 * 1024)
          throw new RangeError('readback exceeds 16 million pixels');
        const floats = format === 'rgba-float16';
        if (floats && typeof Float16Array === 'undefined')
          throw new DOMException('Float16Array unavailable', 'NotSupportedError');
        const target = surface(receiver, canvasWidth, canvasHeight);
        const output = floats
          ? new Float16Array(width * height * 4)
          : new Uint8ClampedArray(width * height * 4);
        for (let row = 0; row < height; row++)
          for (let column = 0; column < width; column++) {
            const sourceX = x + column,
              sourceY = y + row;
            if (sourceX < 0 || sourceY < 0 || sourceX >= canvasWidth || sourceY >= canvasHeight)
              continue;
            const from = (sourceY * canvasWidth + sourceX) * 4;
            const to = (row * width + column) * 4;
            for (let channel = 0; channel < 4; channel++) {
              const value =
                channel === 3 && alpha.get(receiver) === false ? 255 : target.data[from + channel];
              output[to + channel] = floats ? value / 255 : value;
            }
          }
        return floats ? new Uint8Array(output.buffer) : output;
      }
      case 'canvasEncode': {
        const [width, height, type] = [args[0], args[1], args[2]];
        if (!width || !height || type !== 'image/png') return 'data:,';
        if (width * height > 16 * 1024 * 1024) return 'data:,';
        const target = surface(receiver, width, height);
        return pngDataURL(width, height, target.data, alpha.get(receiver) !== false);
      }
      case 'canvasCreateImageBitmap':
        return JSON.stringify({
          id: nextResource++,
          width: args[0],
          height: args[1],
          originClean: true,
        });
      case 'canvasDecodeImageBitmap':
        throw new DOMException('Image decoding is unavailable', 'NotSupportedError');
      case 'imageMetadata':
        return JSON.stringify({ complete: false, width: 0, height: 0, originClean: true });
      default:
        if (operation.startsWith('canvas2d') || operation === 'canvasDestroyImageBitmap')
          return undefined;
        throw new Error(`Unknown Canvas operation: ${operation}`);
    }
  }
  const contexts = new WeakMap();
  const construct = Symbol('CanvasRenderingContext2D construction');
  const identity = () => [1, 0, 0, 1, 0, 0];
  const compositeOperations = new Set([
    'source-over',
    'source-in',
    'source-out',
    'source-atop',
    'destination-over',
    'destination-in',
    'destination-out',
    'destination-atop',
    'lighter',
    'copy',
    'xor',
    'multiply',
    'screen',
    'overlay',
    'darken',
    'lighten',
    'color-dodge',
    'color-burn',
    'hard-light',
    'soft-light',
    'difference',
    'exclusion',
    'hue',
    'saturation',
    'color',
    'luminosity',
  ]);

  const finite = (values) => values.every(Number.isFinite);
  function normalizedSweep(startAngle, endAngle, counterclockwise) {
    const turn = Math.PI * 2;
    let sweep = endAngle - startAngle;
    if (!counterclockwise) {
      if (sweep >= turn) return turn;
      sweep %= turn;
      return sweep < 0 ? sweep + turn : sweep;
    }
    if (-sweep >= turn) return -turn;
    sweep %= turn;
    return sweep > 0 ? sweep - turn : sweep;
  }

  function roundRectRadii(value, width, height) {
    let values;
    if (
      typeof value !== 'object' ||
      value === null ||
      typeof value[Symbol.iterator] !== 'function'
    ) {
      values = [value ?? 0];
    } else {
      values = [...value];
    }
    if (values.length < 1 || values.length > 4)
      throw new RangeError('roundRect requires between one and four radii');
    const points = values.map((radius) => {
      const x = Number(typeof radius === 'object' && radius !== null ? (radius.x ?? 0) : radius);
      const y = Number(typeof radius === 'object' && radius !== null ? (radius.y ?? 0) : radius);
      if (!Number.isFinite(x) || !Number.isFinite(y)) return { x: 0, y: 0 };
      if (x < 0 || y < 0) throw new RangeError('roundRect radii must be non-negative');
      return { x, y };
    });
    let corners;
    if (points.length === 1) corners = [points[0], points[0], points[0], points[0]];
    else if (points.length === 2) corners = [points[0], points[1], points[0], points[1]];
    else if (points.length === 3) corners = [points[0], points[1], points[2], points[1]];
    else corners = points;
    if (width < 0)
      [corners[0], corners[1], corners[2], corners[3]] = [
        corners[1],
        corners[0],
        corners[3],
        corners[2],
      ];
    if (height < 0)
      [corners[0], corners[1], corners[2], corners[3]] = [
        corners[3],
        corners[2],
        corners[1],
        corners[0],
      ];
    const ratios = [
      Math.abs(width) / (corners[0].x + corners[1].x),
      Math.abs(height) / (corners[1].y + corners[2].y),
      Math.abs(width) / (corners[2].x + corners[3].x),
      Math.abs(height) / (corners[3].y + corners[0].y),
    ].filter(Number.isFinite);
    const scale = Math.min(1, ...ratios);
    return corners.flatMap((radius) => [radius.x * scale, radius.y * scale]);
  }

  function matrixValues(matrix = {}) {
    if (matrix?.is2D === false)
      throw new DOMException('The transform must be two-dimensional', 'InvalidStateError');
    const values = [
      matrix.a ?? matrix.m11 ?? 1,
      matrix.b ?? matrix.m12 ?? 0,
      matrix.c ?? matrix.m21 ?? 0,
      matrix.d ?? matrix.m22 ?? 1,
      matrix.e ?? matrix.m41 ?? 0,
      matrix.f ?? matrix.m42 ?? 0,
    ].map(Number);
    if (!finite(values)) throw new TypeError('The transform must contain finite values');
    return values;
  }
  const ensureOriginClean = (canvas) => {
    if (!call('canvasOriginClean', canvas)) {
      throw new DOMException('The canvas contains cross-origin data', 'SecurityError');
    }
  };
  const multiply = (left, right) => [
    left[0] * right[0] + left[2] * right[1],
    left[1] * right[0] + left[3] * right[1],
    left[0] * right[2] + left[2] * right[3],
    left[1] * right[2] + left[3] * right[3],
    left[0] * right[4] + left[2] * right[5] + left[4],
    left[1] * right[4] + left[3] * right[5] + left[5],
  ];

  const namedColors = Object.freeze({
    transparent: [0, 0, 0, 0],
    black: [0, 0, 0, 1],
    silver: [192, 192, 192, 1],
    gray: [128, 128, 128, 1],
    grey: [128, 128, 128, 1],
    white: [255, 255, 255, 1],
    maroon: [128, 0, 0, 1],
    red: [255, 0, 0, 1],
    purple: [128, 0, 128, 1],
    fuchsia: [255, 0, 255, 1],
    green: [0, 128, 0, 1],
    lime: [0, 255, 0, 1],
    olive: [128, 128, 0, 1],
    yellow: [255, 255, 0, 1],
    navy: [0, 0, 128, 1],
    blue: [0, 0, 255, 1],
    teal: [0, 128, 128, 1],
    aqua: [0, 255, 255, 1],
    orange: [255, 165, 0, 1],
    rebeccapurple: [102, 51, 153, 1],
  });

  function parseComponent(value) {
    const text = value.trim();
    const number = Number.parseFloat(text);
    if (!Number.isFinite(number)) return null;
    return Math.max(0, Math.min(255, text.endsWith('%') ? number * 2.55 : number));
  }

  function parseAlpha(value) {
    const text = value.trim();
    const number = Number.parseFloat(text);
    if (!Number.isFinite(number)) return null;
    return Math.max(0, Math.min(1, text.endsWith('%') ? number / 100 : number));
  }

  function parseColorText(value) {
    const text = String(value).trim().toLowerCase();
    const named = namedColors[text];
    if (named)
      return {
        serialized: text,
        rgba: [named[0] / 255, named[1] / 255, named[2] / 255, named[3]],
      };
    const hex = /^#([0-9a-f]{3,4}|[0-9a-f]{6}|[0-9a-f]{8})$/i.exec(text)?.[1];
    if (hex) {
      const expanded =
        hex.length <= 4 ? [...hex].map((character) => character + character).join('') : hex;
      const alpha = expanded.length === 8 ? Number.parseInt(expanded.slice(6, 8), 16) / 255 : 1;
      return {
        serialized: text,
        rgba: [
          Number.parseInt(expanded.slice(0, 2), 16) / 255,
          Number.parseInt(expanded.slice(2, 4), 16) / 255,
          Number.parseInt(expanded.slice(4, 6), 16) / 255,
          alpha,
        ],
      };
    }
    const functional = /^rgba?\((.*)\)$/i.exec(text)?.[1];
    if (functional !== undefined) {
      let components;
      let alpha = '1';
      if (functional.includes(',')) {
        components = functional.split(',').map((value) => value.trim());
        if (components.length === 4) alpha = components.pop();
      } else {
        const parts = functional.split('/');
        components = parts[0].trim().split(/\s+/);
        if (parts.length === 2) alpha = parts[1];
      }
      if (components.length === 3) {
        const rgb = components.map(parseComponent);
        const parsedAlpha = parseAlpha(alpha);
        if (rgb.every((value) => value !== null) && parsedAlpha !== null) {
          return {
            serialized: text,
            rgba: [rgb[0] / 255, rgb[1] / 255, rgb[2] / 255, parsedAlpha],
          };
        }
      }
    }
    return null;
  }

  function parseColor(value) {
    return parseColorText(value);
  }

  const matrixOperation = (matrix) => ({ matrix });
  const amountValue = (text, maximum = Infinity) => {
    text = text.trim();
    if (text === '') return 1;
    const match = /^([+]?(?:\d+(?:\.\d*)?|\.\d+))(%?)$/.exec(text);
    if (!match) return null;
    const value = Number(match[1]);
    const amount = match[2] ? value / 100 : value;
    return Math.min(maximum, amount);
  };
  const colorTransform = (red, green, blue, alpha = [0, 0, 0, 1, 0]) =>
    matrixOperation([...red, ...green, ...blue, ...alpha]);

  function parseCanvasFilter(value) {
    const serialized = String(value).trim();
    if (serialized === 'none') return { serialized: 'none', operations: [] };
    const operations = [];
    const expression = /([a-z-]+)\(([^()]*)\)/gi;
    let end = 0;
    for (let match; (match = expression.exec(serialized));) {
      if (serialized.slice(end, match.index).trim()) return null;
      end = expression.lastIndex;
      const name = match[1].toLowerCase();
      const argument = match[2].trim();
      let amount;
      if (name === 'url') {
        const reference = argument.replace(/^(?:'|")|(?:'|")$/g, '');
        if (!reference) return null;
        operations.push({ svgFilter: reference });
        continue;
      }
      if (name === 'blur') {
        const length = /^([+]?(?:\d+(?:\.\d*)?|\.\d+))(px)?$/i.exec(argument);
        if (!length || (!length[2] && Number(length[1]) !== 0)) return null;
        const sigma = Number(length[1]);
        operations.push({ blur: [sigma, sigma] });
        continue;
      }
      if (name === 'drop-shadow') {
        const lengths = [];
        let color = parseColor('black');
        let sawColor = false;
        for (const token of argument.split(/\s+/).filter(Boolean)) {
          const length = /^([+-]?(?:\d+(?:\.\d*)?|\.\d+))(px)?$/i.exec(token);
          if (length && (length[2] || Number(length[1]) === 0)) {
            lengths.push(Number(length[1]));
            continue;
          }
          const parsed = sawColor ? null : parseColor(token);
          if (!parsed) return null;
          color = parsed;
          sawColor = true;
        }
        if (lengths.length < 2 || lengths.length > 3 || (lengths[2] ?? 0) < 0) return null;
        operations.push({
          dropShadow: {
            offsetX: lengths[0],
            offsetY: lengths[1],
            blur: lengths[2] ?? 0,
            color: color.rgba,
          },
        });
        continue;
      }
      if (name === 'hue-rotate') {
        const angle = /^([+-]?(?:\d+(?:\.\d*)?|\.\d+))(deg|rad|grad|turn)?$/i.exec(
          argument || '0deg',
        );
        if (!angle || (!angle[2] && Number(angle[1]) !== 0)) return null;
        let radians = Number(angle[1]);
        if (angle[2]?.toLowerCase() === 'deg') radians *= Math.PI / 180;
        else if (angle[2]?.toLowerCase() === 'grad') radians *= Math.PI / 200;
        else if (angle[2]?.toLowerCase() === 'turn') radians *= Math.PI * 2;
        const cosine = Math.cos(radians),
          sine = Math.sin(radians);
        operations.push(
          colorTransform(
            [
              0.213 + cosine * 0.787 - sine * 0.213,
              0.715 - cosine * 0.715 - sine * 0.715,
              0.072 - cosine * 0.072 + sine * 0.928,
              0,
              0,
            ],
            [
              0.213 - cosine * 0.213 + sine * 0.143,
              0.715 + cosine * 0.285 + sine * 0.14,
              0.072 - cosine * 0.072 - sine * 0.283,
              0,
              0,
            ],
            [
              0.213 - cosine * 0.213 - sine * 0.787,
              0.715 - cosine * 0.715 + sine * 0.715,
              0.072 + cosine * 0.928 + sine * 0.072,
              0,
              0,
            ],
          ),
        );
        continue;
      }
      const limited = ['grayscale', 'invert', 'opacity', 'sepia'].includes(name);
      amount = amountValue(argument, limited ? 1 : Infinity);
      if (amount === null) return null;
      if (name === 'brightness') {
        operations.push(
          colorTransform([amount, 0, 0, 0, 0], [0, amount, 0, 0, 0], [0, 0, amount, 0, 0]),
        );
      } else if (name === 'contrast') {
        const offset = 0.5 * (1 - amount);
        operations.push(
          colorTransform(
            [amount, 0, 0, 0, offset],
            [0, amount, 0, 0, offset],
            [0, 0, amount, 0, offset],
          ),
        );
      } else if (name === 'opacity') {
        operations.push(
          colorTransform([1, 0, 0, 0, 0], [0, 1, 0, 0, 0], [0, 0, 1, 0, 0], [0, 0, 0, amount, 0]),
        );
      } else if (name === 'invert') {
        const scale = 1 - amount * 2,
          offset = amount;
        operations.push(
          colorTransform(
            [scale, 0, 0, 0, offset],
            [0, scale, 0, 0, offset],
            [0, 0, scale, 0, offset],
          ),
        );
      } else if (name === 'grayscale') {
        const inverse = 1 - amount;
        operations.push(
          colorTransform(
            [inverse + 0.2126 * amount, 0.7152 * amount, 0.0722 * amount, 0, 0],
            [0.2126 * amount, inverse + 0.7152 * amount, 0.0722 * amount, 0, 0],
            [0.2126 * amount, 0.7152 * amount, inverse + 0.0722 * amount, 0, 0],
          ),
        );
      } else if (name === 'saturate') {
        operations.push(
          colorTransform(
            [0.213 + 0.787 * amount, 0.715 - 0.715 * amount, 0.072 - 0.072 * amount, 0, 0],
            [0.213 - 0.213 * amount, 0.715 + 0.285 * amount, 0.072 - 0.072 * amount, 0, 0],
            [0.213 - 0.213 * amount, 0.715 - 0.715 * amount, 0.072 + 0.928 * amount, 0, 0],
          ),
        );
      } else if (name === 'sepia') {
        const inverse = 1 - amount;
        operations.push(
          colorTransform(
            [inverse + 0.393 * amount, 0.769 * amount, 0.189 * amount, 0, 0],
            [0.349 * amount, inverse + 0.686 * amount, 0.168 * amount, 0, 0],
            [0.272 * amount, 0.534 * amount, inverse + 0.131 * amount, 0, 0],
          ),
        );
      } else {
        return null;
      }
    }
    if (end === 0 || serialized.slice(end).trim()) return null;
    return { serialized, operations };
  }

  const svgNumber = (element, name, fallback = 0) => {
    const value = element.getAttribute(name);
    if (value === null || value.trim() === '') return fallback;
    const number = Number(value);
    return Number.isFinite(number) ? number : null;
  };

  function svgLightSource(primitive) {
    let light;
    for (const child of primitive.children) {
      const name = child.localName?.toLowerCase();
      if (['desc', 'title', 'metadata', 'script', 'animate', 'set'].includes(name)) continue;
      if (light !== undefined) return null;
      if (name === 'fedistantlight') {
        const azimuth = svgNumber(child, 'azimuth');
        const elevation = svgNumber(child, 'elevation');
        if (azimuth === null || elevation === null) return null;
        light = { kind: 'distant', azimuth, elevation };
      } else if (name === 'fepointlight') {
        const position = ['x', 'y', 'z'].map((attribute) => svgNumber(child, attribute));
        if (position.some((value) => value === null)) return null;
        light = { kind: 'point', position };
      } else if (name === 'fespotlight') {
        const position = ['x', 'y', 'z'].map((attribute) => svgNumber(child, attribute));
        const target = ['pointsAtX', 'pointsAtY', 'pointsAtZ'].map((attribute) =>
          svgNumber(child, attribute),
        );
        const falloffExponent = svgNumber(child, 'specularExponent', 1);
        const cone = child.getAttribute('limitingConeAngle');
        const cutoffAngle = cone === null || cone.trim() === '' ? 90 : Number(cone);
        if (
          position.some((value) => value === null) ||
          target.some((value) => value === null) ||
          falloffExponent === null ||
          falloffExponent < 1 ||
          falloffExponent > 128 ||
          !Number.isFinite(cutoffAngle) ||
          cutoffAngle < 0 ||
          cutoffAngle > 90
        )
          return null;
        light = { kind: 'spot', position, target, falloffExponent, cutoffAngle };
      } else {
        return null;
      }
    }
    return light ?? null;
  }

  function appendSvgFilter(document, reference, nodes, sourceInput) {
    if (!reference.startsWith('#')) return sourceInput;
    const filter = document?.getElementById(reference.slice(1));
    if (!filter || filter.localName?.toLowerCase() !== 'filter') return sourceInput;
    const start = nodes.length;
    const results = new Map();
    let previous = sourceInput;
    let sourceAlpha;
    const fail = () => {
      nodes.length = start;
      return sourceInput;
    };
    const resolveInput = (element, name, fallback) => {
      const value = element.getAttribute(name)?.trim();
      if (!value) return fallback;
      if (value === 'SourceGraphic') return sourceInput;
      if (value === 'SourceAlpha') {
        if (sourceAlpha === undefined) {
          nodes.push({
            matrix: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0],
            input: sourceInput,
          });
          sourceAlpha = nodes.length - 1;
        }
        return sourceAlpha;
      }
      return results.get(value);
    };
    for (const primitive of filter.children) {
      const name = primitive.localName.toLowerCase();
      const input = resolveInput(primitive, 'in', previous);
      if (input === undefined) return fail();
      let operation;
      if (name === 'fegaussianblur') {
        const values = (primitive.getAttribute('stdDeviation') ?? '0')
          .trim()
          .split(/[ ,]+/)
          .filter(Boolean)
          .map(Number);
        if (
          values.length < 1 ||
          values.length > 2 ||
          values.some((value) => !Number.isFinite(value) || value < 0)
        )
          return fail();
        operation = { blur: [values[0], values[1] ?? values[0]], input };
      } else if (name === 'feoffset') {
        const x = svgNumber(primitive, 'dx'),
          y = svgNumber(primitive, 'dy');
        if (x === null || y === null) return fail();
        operation = { offset: [x, y], input };
      } else if (name === 'fedropshadow') {
        const x = svgNumber(primitive, 'dx'),
          y = svgNumber(primitive, 'dy');
        const sigma = svgNumber(primitive, 'stdDeviation');
        const opacity = svgNumber(primitive, 'flood-opacity', 1);
        const color = parseColor(primitive.getAttribute('flood-color') ?? 'black');
        if (
          x === null ||
          y === null ||
          sigma === null ||
          sigma < 0 ||
          opacity === null ||
          opacity < 0 ||
          !color
        )
          return fail();
        const rgba = color.rgba.slice();
        rgba[3] *= Math.min(1, opacity);
        operation = {
          dropShadow: {
            offsetX: x,
            offsetY: y,
            blur: sigma * 2,
            color: rgba,
          },
          input,
        };
      } else if (name === 'fecolormatrix') {
        const type = (primitive.getAttribute('type') ?? 'matrix').toLowerCase();
        const text = primitive.getAttribute('values') ?? '';
        if (type === 'matrix') {
          const values =
            text.trim() === ''
              ? [1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0]
              : text.trim().split(/[ ,]+/).map(Number);
          if (values.length !== 20 || values.some((value) => !Number.isFinite(value)))
            return fail();
          operation = { ...matrixOperation(values), input };
        } else if (type === 'saturate') {
          const amount = text.trim() === '' ? 1 : Number(text);
          if (!Number.isFinite(amount)) return fail();
          operation = {
            ...colorTransform(
              [0.213 + 0.787 * amount, 0.715 - 0.715 * amount, 0.072 - 0.072 * amount, 0, 0],
              [0.213 - 0.213 * amount, 0.715 + 0.285 * amount, 0.072 - 0.072 * amount, 0, 0],
              [0.213 - 0.213 * amount, 0.715 - 0.715 * amount, 0.072 + 0.928 * amount, 0, 0],
            ),
            input,
          };
        } else if (type === 'huerotate') {
          const angle = text.trim() === '' ? 0 : Number(text);
          if (!Number.isFinite(angle)) return fail();
          const radians = (angle * Math.PI) / 180;
          const cosine = Math.cos(radians),
            sine = Math.sin(radians);
          operation = {
            ...colorTransform(
              [
                0.213 + cosine * 0.787 - sine * 0.213,
                0.715 - cosine * 0.715 - sine * 0.715,
                0.072 - cosine * 0.072 + sine * 0.928,
                0,
                0,
              ],
              [
                0.213 - cosine * 0.213 + sine * 0.143,
                0.715 + cosine * 0.285 + sine * 0.14,
                0.072 - cosine * 0.072 - sine * 0.283,
                0,
                0,
              ],
              [
                0.213 - cosine * 0.213 - sine * 0.787,
                0.715 - cosine * 0.715 + sine * 0.715,
                0.072 + cosine * 0.928 + sine * 0.072,
                0,
                0,
              ],
            ),
            input,
          };
        } else if (type === 'luminancetoalpha') {
          operation = {
            ...colorTransform(
              [0, 0, 0, 0, 0],
              [0, 0, 0, 0, 0],
              [0, 0, 0, 0, 0],
              [0.2125, 0.7154, 0.0721, 0, 0],
            ),
            input,
          };
        } else {
          return fail();
        }
      } else if (name === 'fecomponenttransfer') {
        const functions = Array.from({ length: 4 }, () => ({ type: 'identity' }));
        const channels = new Map([
          ['fefuncr', 0],
          ['fefuncg', 1],
          ['fefuncb', 2],
          ['fefunca', 3],
        ]);
        const seen = new Set();
        for (const transfer of primitive.children) {
          const channel = channels.get(transfer.localName?.toLowerCase());
          if (channel === undefined || seen.has(channel)) return fail();
          seen.add(channel);
          const type = (transfer.getAttribute('type') ?? 'identity').toLowerCase();
          if (type === 'identity') {
            functions[channel] = { type };
          } else if (type === 'table' || type === 'discrete') {
            const values = (transfer.getAttribute('tableValues') ?? '')
              .trim()
              .split(/[ ,]+/)
              .filter(Boolean)
              .map(Number);
            if (values.length === 0 || values.some((value) => !Number.isFinite(value)))
              return fail();
            functions[channel] = { type, values };
          } else if (type === 'linear') {
            const slope = svgNumber(transfer, 'slope', 1);
            const intercept = svgNumber(transfer, 'intercept');
            if (slope === null || intercept === null) return fail();
            functions[channel] = { type, slope, intercept };
          } else if (type === 'gamma') {
            const amplitude = svgNumber(transfer, 'amplitude', 1);
            const exponent = svgNumber(transfer, 'exponent', 1);
            const offset = svgNumber(transfer, 'offset');
            if (amplitude === null || exponent === null || offset === null) return fail();
            functions[channel] = { type, amplitude, exponent, offset };
          } else {
            return fail();
          }
        }
        operation = { componentTransfer: functions, input };
      } else if (name === 'femorphology') {
        const operator = (primitive.getAttribute('operator') ?? 'erode').toLowerCase();
        if (operator !== 'erode' && operator !== 'dilate') return fail();
        const radius = (primitive.getAttribute('radius') ?? '0')
          .trim()
          .split(/[ ,]+/)
          .filter(Boolean)
          .map(Number);
        if (
          radius.length < 1 ||
          radius.length > 2 ||
          radius.some((value) => !Number.isFinite(value) || value < 0)
        )
          return fail();
        operation = {
          morphology: {
            operator,
            radius: [radius[0], radius[1] ?? radius[0]],
          },
          input,
        };
      } else if (name === 'feflood') {
        const opacity = svgNumber(primitive, 'flood-opacity', 1);
        const color = parseColor(primitive.getAttribute('flood-color') ?? 'black');
        if (opacity === null || opacity < 0 || !color) return fail();
        const rgba = color.rgba.slice();
        rgba[3] *= Math.min(1, opacity);
        operation = { flood: rgba, input };
      } else if (name === 'feconvolvematrix') {
        const order = (primitive.getAttribute('order') ?? '3')
          .trim()
          .split(/[ ,]+/)
          .filter(Boolean)
          .map(Number);
        if (order.length < 1 || order.length > 2 || order.some((value) => !Number.isFinite(value)))
          return fail();
        const width = Math.trunc(order[0]),
          height = Math.trunc(order[1] ?? order[0]);
        if (width <= 0 || height <= 0 || width > 64 || height > 64) return fail();
        const kernel = (primitive.getAttribute('kernelMatrix') ?? '')
          .trim()
          .split(/[ ,]+/)
          .filter(Boolean)
          .map(Number);
        if (kernel.some((value) => !Number.isFinite(value))) return fail();
        if (kernel.length !== width * height) {
          operation = {
            ...matrixOperation([1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0]),
            input,
          };
        } else {
          const defaultDivisor = kernel.reduce((sum, value) => sum + value, 0) || 1;
          const divisor = svgNumber(primitive, 'divisor', defaultDivisor);
          const bias = svgNumber(primitive, 'bias');
          const targetX = svgNumber(primitive, 'targetX', Math.floor(width / 2));
          const targetY = svgNumber(primitive, 'targetY', Math.floor(height / 2));
          const edgeMode = (primitive.getAttribute('edgeMode') ?? 'duplicate').toLowerCase();
          const preserveAlpha = (primitive.getAttribute('preserveAlpha') ?? 'false').toLowerCase();
          const unitLength = primitive.getAttribute('kernelUnitLength');
          const units =
            unitLength === null
              ? [1]
              : unitLength.trim().split(/[ ,]+/).filter(Boolean).map(Number);
          const gain = 1 / (divisor || defaultDivisor);
          if (
            !Number.isFinite(defaultDivisor) ||
            !Number.isFinite(gain) ||
            divisor === null ||
            bias === null ||
            targetX === null ||
            targetY === null ||
            targetX !== Math.trunc(targetX) ||
            targetY !== Math.trunc(targetY) ||
            targetX < 0 ||
            targetX >= width ||
            targetY < 0 ||
            targetY >= height ||
            !['duplicate', 'wrap', 'none'].includes(edgeMode) ||
            !['true', 'false'].includes(preserveAlpha) ||
            units.length < 1 ||
            units.length > 2 ||
            units.some((value) => value !== 1)
          )
            return fail();
          operation = {
            convolveMatrix: {
              order: [width, height],
              kernel,
              gain,
              bias,
              target: [targetX, targetY],
              edgeMode,
              convolveAlpha: preserveAlpha !== 'true',
            },
            input,
          };
        }
      } else if (name === 'fedisplacementmap') {
        const input2 = resolveInput(primitive, 'in2', previous);
        const scale = svgNumber(primitive, 'scale');
        const xChannel = (primitive.getAttribute('xChannelSelector') ?? 'A').toUpperCase();
        const yChannel = (primitive.getAttribute('yChannelSelector') ?? 'A').toUpperCase();
        if (
          input2 === undefined ||
          scale === null ||
          !['R', 'G', 'B', 'A'].includes(xChannel) ||
          !['R', 'G', 'B', 'A'].includes(yChannel)
        )
          return fail();
        operation = { displacementMap: { scale, xChannel, yChannel }, input, input2 };
      } else if (name === 'fediffuselighting' || name === 'fespecularlighting') {
        const specular = name === 'fespecularlighting';
        const surfaceScale = svgNumber(primitive, 'surfaceScale', 1);
        const constant = svgNumber(primitive, specular ? 'specularConstant' : 'diffuseConstant', 1);
        const exponent = specular ? svgNumber(primitive, 'specularExponent', 1) : 1;
        const color = parseColor(primitive.getAttribute('lighting-color') ?? 'white');
        const light = svgLightSource(primitive);
        const unitLength = primitive.getAttribute('kernelUnitLength');
        const units =
          unitLength === null ? [1] : unitLength.trim().split(/[ ,]+/).filter(Boolean).map(Number);
        if (
          surfaceScale === null ||
          constant === null ||
          constant < 0 ||
          exponent === null ||
          exponent < 1 ||
          exponent > 128 ||
          !color ||
          !light ||
          units.length < 1 ||
          units.length > 2 ||
          units.some((value) => value !== 1)
        )
          return fail();
        operation = {
          lighting: {
            specular,
            color: color.rgba.slice(0, 3),
            surfaceScale,
            constant,
            exponent,
            light,
          },
          input,
        };
      } else if (name === 'feblend') {
        const mode = (primitive.getAttribute('mode') ?? 'normal').toLowerCase();
        if (
          ![
            'normal',
            'multiply',
            'screen',
            'darken',
            'lighten',
            'overlay',
            'color-dodge',
            'color-burn',
            'hard-light',
            'soft-light',
            'difference',
            'exclusion',
            'hue',
            'saturation',
            'color',
            'luminosity',
          ].includes(mode)
        )
          return fail();
        const input2 = resolveInput(primitive, 'in2', previous);
        if (input2 === undefined) return fail();
        operation = { blend: mode, input, input2 };
      } else if (name === 'fecomposite') {
        const operator = (primitive.getAttribute('operator') ?? 'over').toLowerCase();
        if (!['over', 'in', 'out', 'atop', 'xor', 'arithmetic'].includes(operator)) return fail();
        const input2 = resolveInput(primitive, 'in2', previous);
        if (input2 === undefined) return fail();
        const coefficients = ['k1', 'k2', 'k3', 'k4'].map((attribute) =>
          svgNumber(primitive, attribute),
        );
        if (coefficients.some((value) => value === null)) return fail();
        operation = { composite: { operator, coefficients }, input, input2 };
      } else if (name === 'femerge') {
        const inputs = [];
        for (const mergeNode of primitive.children) {
          if (mergeNode.localName?.toLowerCase() !== 'femergenode') return fail();
          const mergeInput = resolveInput(mergeNode, 'in', previous);
          if (mergeInput === undefined) return fail();
          inputs.push(mergeInput);
        }
        if (inputs.length === 0) return fail();
        operation = { merge: inputs, input };
      } else {
        return fail();
      }
      nodes.push(operation);
      previous = nodes.length - 1;
      const result = primitive.getAttribute('result')?.trim();
      if (result) results.set(result, previous);
    }
    return previous;
  }

  class CanvasGradient {
    constructor(token, canvas, id) {
      if (token !== construct) throw new TypeError('Illegal constructor');
      Object.defineProperties(this, { __canvas: { value: canvas }, __id: { value: id } });
    }
    addColorStop(offset, color) {
      offset = Number(offset);
      if (!Number.isFinite(offset) || offset < 0 || offset > 1) {
        throw new DOMException(
          'The color stop offset must be between zero and one',
          'IndexSizeError',
        );
      }
      const parsed = parseColor(color);
      if (!parsed) throw new DOMException('The color could not be parsed', 'SyntaxError');
      call('canvas2dAddColorStop', this.__canvas, this.__id, offset, ...parsed.rgba);
    }
  }

  class CanvasPattern {
    constructor(token, canvas, id) {
      if (token !== construct) throw new TypeError('Illegal constructor');
      Object.defineProperties(this, { __canvas: { value: canvas }, __id: { value: id } });
    }
    setTransform(matrix = {}) {
      const values = matrixValues(matrix);
      call('canvas2dSetPatternTransform', this.__canvas, this.__id, ...values);
    }
  }

  class Path2D {
    constructor(source = undefined) {
      let id;
      if (source === undefined) id = call('canvas2dCreatePath', globalThis, 'empty', 0);
      else if (source instanceof Path2D)
        id = call('canvas2dCreatePath', globalThis, 'copy', source.__id);
      else id = call('canvas2dCreatePath', globalThis, 'svg', String(source));
      Object.defineProperty(this, '__id', { value: id });
    }
    addPath(path, transform = {}) {
      if (!(path instanceof Path2D)) throw new TypeError('addPath expects a Path2D');
      call('canvas2dAddPath', globalThis, this.__id, path.__id, ...matrixValues(transform));
    }
    closePath() {
      call('canvas2dPath2DClose', globalThis, this.__id);
    }
    moveTo(x, y) {
      call('canvas2dPath2DPoint', globalThis, this.__id, 'move', Number(x), Number(y));
    }
    lineTo(x, y) {
      call('canvas2dPath2DPoint', globalThis, this.__id, 'line', Number(x), Number(y));
    }
    quadraticCurveTo(controlX, controlY, x, y) {
      call(
        'canvas2dPath2DPoint',
        globalThis,
        this.__id,
        'quadratic',
        Number(controlX),
        Number(controlY),
        Number(x),
        Number(y),
      );
    }
    bezierCurveTo(firstX, firstY, secondX, secondY, x, y) {
      call(
        'canvas2dPath2DPoint',
        globalThis,
        this.__id,
        'bezier',
        Number(firstX),
        Number(firstY),
        Number(secondX),
        Number(secondY),
        Number(x),
        Number(y),
      );
    }
    arcTo(firstX, firstY, secondX, secondY, radius) {
      [firstX, firstY, secondX, secondY, radius] = [firstX, firstY, secondX, secondY, radius].map(
        Number,
      );
      if (radius < 0) throw new DOMException('The radius provided is negative', 'IndexSizeError');
      if (!finite([firstX, firstY, secondX, secondY, radius])) return;
      call('canvas2dPath2DArcTo', globalThis, this.__id, firstX, firstY, secondX, secondY, radius);
    }
    rect(x, y, width, height) {
      call(
        'canvas2dPath2DRect',
        globalThis,
        this.__id,
        Number(x),
        Number(y),
        Number(width),
        Number(height),
      );
    }
    roundRect(x, y, width, height, radii = 0) {
      [x, y, width, height] = [x, y, width, height].map(Number);
      if (!finite([x, y, width, height])) return;
      const corners = roundRectRadii(radii, width, height);
      if (width < 0) {
        x += width;
        width = -width;
      }
      if (height < 0) {
        y += height;
        height = -height;
      }
      call('canvas2dPath2DRoundRect', globalThis, this.__id, x, y, width, height, ...corners);
    }
    arc(x, y, radius, startAngle, endAngle, counterclockwise = false) {
      [x, y, radius, startAngle, endAngle] = [x, y, radius, startAngle, endAngle].map(Number);
      if (radius < 0) throw new DOMException('The radius provided is negative', 'IndexSizeError');
      if (!finite([x, y, radius, startAngle, endAngle])) return;
      call(
        'canvas2dPath2DArc',
        globalThis,
        this.__id,
        x,
        y,
        radius,
        startAngle,
        normalizedSweep(startAngle, endAngle, Boolean(counterclockwise)),
      );
    }
    ellipse(x, y, radiusX, radiusY, rotation, startAngle, endAngle, counterclockwise = false) {
      [x, y, radiusX, radiusY, rotation, startAngle, endAngle] = [
        x,
        y,
        radiusX,
        radiusY,
        rotation,
        startAngle,
        endAngle,
      ].map(Number);
      if (radiusX < 0 || radiusY < 0)
        throw new DOMException('The radius provided is negative', 'IndexSizeError');
      if (!finite([x, y, radiusX, radiusY, rotation, startAngle, endAngle])) return;
      call(
        'canvas2dPath2DEllipse',
        globalThis,
        this.__id,
        x,
        y,
        radiusX,
        radiusY,
        rotation,
        startAngle,
        normalizedSweep(startAngle, endAngle, Boolean(counterclockwise)),
      );
    }
  }

  function stylePayload(style, alpha) {
    if (style instanceof CanvasGradient) return JSON.stringify({ gradient: style.__id, alpha });
    if (style instanceof CanvasPattern) return JSON.stringify({ pattern: style.__id, alpha });
    const rgba = style.rgba.slice();
    rgba[3] *= alpha;
    return JSON.stringify({ color: rgba });
  }

  const strokePayload = (context) =>
    JSON.stringify({
      width: context.__lineWidth,
      cap: context.__lineCap,
      join: context.__lineJoin,
      miterLimit: context.__miterLimit,
      dash: context.__lineDash,
      dashOffset: context.__lineDashOffset,
    });
  const effectsPayload = (context) => {
    const color = context.__shadowColor.rgba.slice();
    color[3] *= context.__globalAlpha;
    const filters = [];
    let input = -1;
    for (const operation of context.__filter.operations) {
      if (operation.svgFilter === undefined) {
        filters.push({ ...operation, input });
        input = filters.length - 1;
      } else {
        input = appendSvgFilter(
          context.canvas.ownerDocument ?? document,
          operation.svgFilter,
          filters,
          input,
        );
      }
    }
    return JSON.stringify({
      shadow: {
        color,
        blur: context.__shadowBlur,
        offsetX: context.__shadowOffsetX,
        offsetY: context.__shadowOffsetY,
      },
      filters,
    });
  };

  const normalizeImageDataSettings = (
    settings,
    defaultColorSpace = 'srgb',
    defaultPixelFormat = 'rgba-unorm8',
  ) => {
    const colorSpace =
      settings?.colorSpace === undefined ? defaultColorSpace : String(settings.colorSpace);
    const pixelFormat =
      settings?.pixelFormat === undefined ? defaultPixelFormat : String(settings.pixelFormat);
    if (colorSpace !== 'srgb' && colorSpace !== 'display-p3')
      throw new TypeError('Invalid ImageData colorSpace');
    if (pixelFormat !== 'rgba-unorm8' && pixelFormat !== 'rgba-float16')
      throw new TypeError('Invalid ImageData pixelFormat');
    return { colorSpace, pixelFormat };
  };

  class ImageBitmap {
    constructor(token, metadata) {
      if (token !== construct) throw new TypeError('Illegal constructor');
      Object.defineProperties(this, {
        __id: { value: Number(metadata.id), writable: true },
        __originClean: { value: metadata.originClean !== false },
        width: { value: Number(metadata.width) >>> 0, enumerable: true },
        height: { value: Number(metadata.height) >>> 0, enumerable: true },
      });
    }
    close() {
      if (this.__id === 0) return;
      call('canvasDestroyImageBitmap', globalThis, this.__id);
      this.__id = 0;
    }
  }

  class TextMetrics {
    constructor(token, values) {
      if (token !== construct) throw new TypeError('Illegal constructor');
      Object.assign(this, values);
    }
  }

  function fontSize(font) {
    return Number(/(?:^|\s)(\d+(?:\.\d+)?)px(?:\s|\/)/.exec(` ${font} `)?.[1] ?? 10);
  }

  function fontFamily(font) {
    const match = /\d+(?:\.\d+)?px(?:\s*\/\s*[^\s]+)?\s+(.+)$/.exec(font);
    const families = (match?.[1] ?? 'sans-serif').split(',');
    for (let family of families) {
      family = family
        .trim()
        .replace(/^(?:'|")|(?:'|")$/g, '')
        .toLowerCase();
      if (['wenquanyi micro hei mono', 'monospace', 'ui-monospace'].includes(family))
        return 'monospace';
      if (['noto color emoji', 'emoji'].includes(family)) return 'emoji';
      if (
        ['wenquanyi micro hei', 'sans-serif', 'serif', 'system-ui', 'cursive', 'fantasy'].includes(
          family,
        )
      )
        return 'proportional';
    }
    return 'proportional';
  }

  function fontFamilyName(font) {
    const match = /\d+(?:\.\d+)?px(?:\s*\/\s*[^\s]+)?\s+(.+)$/.exec(font);
    return (match?.[1] ?? 'sans-serif')
      .split(',')[0]
      .trim()
      .replace(/^(?:'|")|(?:'|")$/g, '')
      .toLowerCase();
  }

  function resolvedTextDirection(context) {
    return context.__direction === 'inherit'
      ? context.canvas.closest('[dir]')?.getAttribute('dir') === 'rtl'
        ? 'rtl'
        : 'ltr'
      : context.__direction;
  }

  class CanvasRenderingContext2D {
    constructor(token, canvas, attributes) {
      if (token !== construct) throw new TypeError('Illegal constructor');
      this.canvas = canvas;
      Object.defineProperty(this, '__attributes', {
        value: Object.freeze({ ...attributes }),
      });
      this.__resetState();
    }
    __resetState() {
      this.__fillStyle = { serialized: '#000000', rgba: [0, 0, 0, 1] };
      this.__strokeStyle = { serialized: '#000000', rgba: [0, 0, 0, 1] };
      this.__globalAlpha = 1;
      this.__globalCompositeOperation = 'source-over';
      this.__lineWidth = 1;
      this.__lineCap = 'butt';
      this.__lineJoin = 'miter';
      this.__miterLimit = 10;
      this.__lineDash = [];
      this.__lineDashOffset = 0;
      this.__shadowColor = { serialized: 'rgba(0, 0, 0, 0)', rgba: [0, 0, 0, 0] };
      this.__shadowBlur = 0;
      this.__shadowOffsetX = 0;
      this.__shadowOffsetY = 0;
      this.__filter = { serialized: 'none', operations: [] };
      this.__imageSmoothingEnabled = true;
      this.__imageSmoothingQuality = 'low';
      this.__font = '10px sans-serif';
      this.__textAlign = 'start';
      this.__textBaseline = 'alphabetic';
      this.__direction = 'inherit';
      this.__transform = identity();
      this.__stack = [];
      call('canvas2dBeginPath', this.canvas, this.canvas.width, this.canvas.height);
    }
    get fillStyle() {
      return this.__fillStyle instanceof CanvasGradient || this.__fillStyle instanceof CanvasPattern
        ? this.__fillStyle
        : this.__fillStyle.serialized;
    }
    set fillStyle(value) {
      if (value instanceof CanvasGradient || value instanceof CanvasPattern) {
        this.__fillStyle = value;
        return;
      }
      const color = parseColor(value);
      if (color) this.__fillStyle = color;
    }
    get strokeStyle() {
      return this.__strokeStyle instanceof CanvasGradient ||
        this.__strokeStyle instanceof CanvasPattern
        ? this.__strokeStyle
        : this.__strokeStyle.serialized;
    }
    set strokeStyle(value) {
      if (value instanceof CanvasGradient || value instanceof CanvasPattern) {
        this.__strokeStyle = value;
        return;
      }
      const color = parseColor(value);
      if (color) this.__strokeStyle = color;
    }
    get globalAlpha() {
      return this.__globalAlpha;
    }
    set globalAlpha(value) {
      value = Number(value);
      if (Number.isFinite(value) && value >= 0 && value <= 1) this.__globalAlpha = value;
    }
    get lineWidth() {
      return this.__lineWidth;
    }
    set lineWidth(value) {
      value = Number(value);
      if (Number.isFinite(value) && value > 0) this.__lineWidth = value;
    }
    get lineCap() {
      return this.__lineCap;
    }
    set lineCap(value) {
      value = String(value);
      if (['butt', 'round', 'square'].includes(value)) this.__lineCap = value;
    }
    get lineJoin() {
      return this.__lineJoin;
    }
    set lineJoin(value) {
      value = String(value);
      if (['round', 'bevel', 'miter'].includes(value)) this.__lineJoin = value;
    }
    get miterLimit() {
      return this.__miterLimit;
    }
    set miterLimit(value) {
      value = Number(value);
      if (Number.isFinite(value) && value > 0) this.__miterLimit = value;
    }
    get lineDashOffset() {
      return this.__lineDashOffset;
    }
    set lineDashOffset(value) {
      value = Number(value);
      if (Number.isFinite(value)) this.__lineDashOffset = value;
    }
    setLineDash(segments) {
      const values = Array.from(segments, Number);
      if (!values.every((value) => Number.isFinite(value) && value >= 0)) {
        throw new DOMException(
          'Line dash values must be finite and non-negative',
          'IndexSizeError',
        );
      }
      this.__lineDash = values.every((value) => value === 0)
        ? []
        : values.length % 2 === 0
          ? values
          : values.concat(values);
    }
    getLineDash() {
      return this.__lineDash.slice();
    }
    get shadowColor() {
      return this.__shadowColor.serialized;
    }
    set shadowColor(value) {
      const color = parseColor(value);
      if (color) this.__shadowColor = color;
    }
    get shadowBlur() {
      return this.__shadowBlur;
    }
    set shadowBlur(value) {
      value = Number(value);
      if (Number.isFinite(value) && value >= 0) this.__shadowBlur = value;
    }
    get shadowOffsetX() {
      return this.__shadowOffsetX;
    }
    set shadowOffsetX(value) {
      value = Number(value);
      if (Number.isFinite(value)) this.__shadowOffsetX = value;
    }
    get shadowOffsetY() {
      return this.__shadowOffsetY;
    }
    set shadowOffsetY(value) {
      value = Number(value);
      if (Number.isFinite(value)) this.__shadowOffsetY = value;
    }
    get filter() {
      return this.__filter.serialized;
    }
    set filter(value) {
      const parsed = parseCanvasFilter(value);
      if (parsed) this.__filter = parsed;
    }
    get globalCompositeOperation() {
      return this.__globalCompositeOperation;
    }
    set globalCompositeOperation(value) {
      value = String(value);
      if (compositeOperations.has(value)) this.__globalCompositeOperation = value;
    }
    get imageSmoothingEnabled() {
      return this.__imageSmoothingEnabled;
    }
    set imageSmoothingEnabled(value) {
      this.__imageSmoothingEnabled = Boolean(value);
    }
    get imageSmoothingQuality() {
      return this.__imageSmoothingQuality;
    }
    set imageSmoothingQuality(value) {
      value = String(value);
      if (value === 'low' || value === 'medium' || value === 'high')
        this.__imageSmoothingQuality = value;
    }
    get font() {
      return this.__font;
    }
    set font(value) {
      const probe = document.createElement('span');
      probe.style.font = String(value);
      if (probe.style.font) this.__font = probe.style.font;
    }
    get textAlign() {
      return this.__textAlign;
    }
    set textAlign(value) {
      value = String(value);
      if (['start', 'end', 'left', 'right', 'center'].includes(value)) this.__textAlign = value;
    }
    get textBaseline() {
      return this.__textBaseline;
    }
    set textBaseline(value) {
      value = String(value);
      if (['top', 'hanging', 'middle', 'alphabetic', 'ideographic', 'bottom'].includes(value))
        this.__textBaseline = value;
    }
    get direction() {
      return this.__direction;
    }
    set direction(value) {
      value = String(value);
      if (['inherit', 'ltr', 'rtl'].includes(value)) this.__direction = value;
    }
    save() {
      call('canvas2dSave', this.canvas, this.canvas.width, this.canvas.height);
      this.__stack.push({
        fillStyle: this.__fillStyle,
        strokeStyle: this.__strokeStyle,
        globalAlpha: this.__globalAlpha,
        lineWidth: this.__lineWidth,
        lineCap: this.__lineCap,
        lineJoin: this.__lineJoin,
        miterLimit: this.__miterLimit,
        lineDash: this.__lineDash.slice(),
        lineDashOffset: this.__lineDashOffset,
        shadowColor: this.__shadowColor,
        shadowBlur: this.__shadowBlur,
        shadowOffsetX: this.__shadowOffsetX,
        shadowOffsetY: this.__shadowOffsetY,
        filter: this.__filter,
        globalCompositeOperation: this.__globalCompositeOperation,
        imageSmoothingEnabled: this.__imageSmoothingEnabled,
        imageSmoothingQuality: this.__imageSmoothingQuality,
        font: this.__font,
        textAlign: this.__textAlign,
        textBaseline: this.__textBaseline,
        direction: this.__direction,
        transform: this.__transform.slice(),
      });
    }
    restore() {
      const state = this.__stack.pop();
      if (!state) return;
      call('canvas2dRestore', this.canvas, this.canvas.width, this.canvas.height);
      this.__fillStyle = state.fillStyle;
      this.__strokeStyle = state.strokeStyle;
      this.__globalAlpha = state.globalAlpha;
      this.__lineWidth = state.lineWidth;
      this.__lineCap = state.lineCap;
      this.__lineJoin = state.lineJoin;
      this.__miterLimit = state.miterLimit;
      this.__lineDash = state.lineDash;
      this.__lineDashOffset = state.lineDashOffset;
      this.__shadowColor = state.shadowColor;
      this.__shadowBlur = state.shadowBlur;
      this.__shadowOffsetX = state.shadowOffsetX;
      this.__shadowOffsetY = state.shadowOffsetY;
      this.__filter = state.filter;
      this.__globalCompositeOperation = state.globalCompositeOperation;
      this.__imageSmoothingEnabled = state.imageSmoothingEnabled;
      this.__imageSmoothingQuality = state.imageSmoothingQuality;
      this.__font = state.font;
      this.__textAlign = state.textAlign;
      this.__textBaseline = state.textBaseline;
      this.__direction = state.direction;
      this.__transform = state.transform;
    }
    reset() {
      call('canvasReset', this.canvas, this.canvas.width, this.canvas.height);
      this.__resetState();
    }
    isContextLost() {
      return false;
    }
    getContextAttributes() {
      return { ...this.__attributes };
    }
    scale(x, y) {
      x = Number(x);
      y = Number(y);
      if (finite([x, y])) this.__transform = multiply(this.__transform, [x, 0, 0, y, 0, 0]);
    }
    rotate(angle) {
      angle = Number(angle);
      if (!Number.isFinite(angle)) return;
      const cosine = Math.cos(angle);
      const sine = Math.sin(angle);
      this.__transform = multiply(this.__transform, [cosine, sine, -sine, cosine, 0, 0]);
    }
    translate(x, y) {
      x = Number(x);
      y = Number(y);
      if (finite([x, y])) this.__transform = multiply(this.__transform, [1, 0, 0, 1, x, y]);
    }
    transform(a, b, c, d, e, f) {
      const matrix = [a, b, c, d, e, f].map(Number);
      if (finite(matrix)) this.__transform = multiply(this.__transform, matrix);
    }
    setTransform(a, b, c, d, e, f) {
      if (typeof a === 'object' && a !== null) {
        ({ a, b, c, d, e, f } = a);
      }
      const matrix = [a, b, c, d, e, f].map(Number);
      if (finite(matrix)) this.__transform = matrix;
    }
    resetTransform() {
      this.__transform = identity();
    }
    getTransform() {
      const [a, b, c, d, e, f] = this.__transform;
      return Object.freeze({
        a,
        b,
        c,
        d,
        e,
        f,
        m11: a,
        m12: b,
        m21: c,
        m22: d,
        m41: e,
        m42: f,
        is2D: true,
      });
    }
    // Legacy callable placeholder: there is no focus-ring renderer.
    drawFocusIfNeeded() {}
    fillRect(x, y, width, height) {
      const values = [x, y, width, height].map(Number);
      call(
        'canvas2dFillRect',
        this.canvas,
        this.canvas.width,
        this.canvas.height,
        ...values,
        stylePayload(this.__fillStyle, this.__globalAlpha),
        ...this.__transform,
        effectsPayload(this),
        this.__globalCompositeOperation,
      );
    }
    strokeRect(x, y, width, height) {
      const values = [x, y, width, height].map(Number);
      call(
        'canvas2dStrokeRect',
        this.canvas,
        this.canvas.width,
        this.canvas.height,
        ...values,
        stylePayload(this.__strokeStyle, this.__globalAlpha),
        ...this.__transform,
        strokePayload(this),
        effectsPayload(this),
        this.__globalCompositeOperation,
      );
    }
    clearRect(x, y, width, height) {
      const values = [x, y, width, height].map(Number);
      call(
        'canvas2dClearRect',
        this.canvas,
        this.canvas.width,
        this.canvas.height,
        ...values,
        ...this.__transform,
      );
    }
    drawImage(source, ...arguments_) {
      const canvasSource = source instanceof HTMLCanvasElement;
      const imageSource = source instanceof HTMLImageElement;
      const bitmapSource = source instanceof ImageBitmap;
      if (!canvasSource && !imageSource && !bitmapSource)
        throw new TypeError('Unsupported CanvasImageSource');
      if (bitmapSource && source.__id === 0)
        throw new DOMException('The ImageBitmap is closed', 'InvalidStateError');
      const metadata = imageSource ? JSON.parse(call('imageMetadata', source)) : null;
      if (imageSource && (!metadata.complete || metadata.width === 0 || metadata.height === 0)) {
        throw new DOMException('The image has no decoded image data', 'InvalidStateError');
      }
      let sourceX = 0;
      let sourceY = 0;
      let sourceWidth = canvasSource || bitmapSource ? source.width : metadata.width;
      let sourceHeight = canvasSource || bitmapSource ? source.height : metadata.height;
      let destinationX;
      let destinationY;
      let destinationWidth;
      let destinationHeight;
      if (arguments_.length === 2) {
        [destinationX, destinationY] = arguments_;
        destinationWidth = sourceWidth;
        destinationHeight = sourceHeight;
      } else if (arguments_.length === 4) {
        [destinationX, destinationY, destinationWidth, destinationHeight] = arguments_;
      } else if (arguments_.length === 8) {
        [
          sourceX,
          sourceY,
          sourceWidth,
          sourceHeight,
          destinationX,
          destinationY,
          destinationWidth,
          destinationHeight,
        ] = arguments_;
      } else {
        throw new TypeError('drawImage requires 3, 5, or 9 arguments');
      }
      const rectangles = [
        sourceX,
        sourceY,
        sourceWidth,
        sourceHeight,
        destinationX,
        destinationY,
        destinationWidth,
        destinationHeight,
      ].map(Number);
      if (!finite(rectangles)) return;
      if (rectangles[2] === 0 || rectangles[3] === 0)
        throw new DOMException('The source width or height is zero', 'IndexSizeError');
      if (canvasSource) {
        call(
          'canvas2dDrawCanvas',
          this.canvas,
          source,
          this.canvas.width,
          this.canvas.height,
          source.width,
          source.height,
          ...rectangles,
          this.__globalAlpha,
          ...this.__transform,
          this.__imageSmoothingEnabled,
          effectsPayload(this),
          this.__globalCompositeOperation,
        );
      } else if (imageSource) {
        call(
          'canvas2dDrawImage',
          this.canvas,
          source,
          this.canvas.width,
          this.canvas.height,
          ...rectangles,
          this.__globalAlpha,
          ...this.__transform,
          this.__imageSmoothingEnabled,
          effectsPayload(this),
          this.__globalCompositeOperation,
          metadata.originClean,
        );
      } else {
        call(
          'canvas2dDrawImageBitmap',
          this.canvas,
          source.__id,
          this.canvas.width,
          this.canvas.height,
          ...rectangles,
          this.__globalAlpha,
          ...this.__transform,
          this.__imageSmoothingEnabled,
          effectsPayload(this),
          this.__globalCompositeOperation,
        );
      }
    }
    createLinearGradient(x0, y0, x1, y1) {
      const coordinates = [x0, y0, x1, y1].map(Number);
      if (!finite(coordinates))
        throw new DOMException('Gradient coordinates must be finite', 'NotSupportedError');
      return new CanvasGradient(
        construct,
        this.canvas,
        call('canvas2dCreateGradient', this.canvas, 'linear', ...coordinates, ...this.__transform),
      );
    }
    createRadialGradient(x0, y0, radius0, x1, y1, radius1) {
      const coordinates = [x0, y0, radius0, x1, y1, radius1].map(Number);
      if (!finite(coordinates))
        throw new DOMException('Gradient coordinates must be finite', 'NotSupportedError');
      if (coordinates[2] < 0 || coordinates[5] < 0)
        throw new DOMException('Gradient radii must not be negative', 'IndexSizeError');
      return new CanvasGradient(
        construct,
        this.canvas,
        call('canvas2dCreateGradient', this.canvas, 'radial', ...coordinates, ...this.__transform),
      );
    }
    createConicGradient(startAngle, x, y) {
      const coordinates = [startAngle, x, y].map(Number);
      if (!finite(coordinates))
        throw new DOMException('Gradient coordinates must be finite', 'NotSupportedError');
      return new CanvasGradient(
        construct,
        this.canvas,
        call('canvas2dCreateGradient', this.canvas, 'conic', ...coordinates, ...this.__transform),
      );
    }
    createPattern(source, repetition = 'repeat') {
      const canvasSource = source instanceof HTMLCanvasElement;
      const imageSource = source instanceof HTMLImageElement;
      const bitmapSource = source instanceof ImageBitmap;
      if (!canvasSource && !imageSource && !bitmapSource)
        throw new TypeError('Unsupported CanvasImageSource');
      if (bitmapSource && source.__id === 0)
        throw new DOMException('The ImageBitmap is closed', 'InvalidStateError');
      const metadata = imageSource ? JSON.parse(call('imageMetadata', source)) : null;
      const sourceWidth = canvasSource || bitmapSource ? source.width : metadata.width;
      const sourceHeight = canvasSource || bitmapSource ? source.height : metadata.height;
      if (sourceWidth === 0 || sourceHeight === 0) return null;
      repetition = repetition === null || repetition === '' ? 'repeat' : String(repetition);
      if (!['repeat', 'repeat-x', 'repeat-y', 'no-repeat'].includes(repetition)) {
        throw new DOMException('Invalid pattern repetition', 'SyntaxError');
      }
      const id = canvasSource
        ? call(
            'canvas2dCreatePattern',
            this.canvas,
            source,
            source.width,
            source.height,
            repetition,
          )
        : imageSource
          ? call(
              'canvas2dCreateImagePattern',
              this.canvas,
              source,
              repetition,
              metadata.originClean,
            )
          : call('canvas2dCreateImageBitmapPattern', this.canvas, source.__id, repetition);
      return new CanvasPattern(construct, this.canvas, id);
    }
    measureText(text) {
      const values = JSON.parse(
        call(
          'canvas2dMeasureText',
          this.canvas,
          String(text),
          fontSize(this.__font),
          fontFamilyName(this.__font),
          resolvedTextDirection(this),
        ),
      );
      return new TextMetrics(construct, {
        width: values[0],
        actualBoundingBoxLeft: values[1],
        actualBoundingBoxRight: values[2],
        actualBoundingBoxAscent: values[3],
        actualBoundingBoxDescent: values[4],
        fontBoundingBoxAscent: values[5],
        fontBoundingBoxDescent: values[6],
        emHeightAscent: values[5],
        emHeightDescent: values[6],
        hangingBaseline: values[5] * 0.8,
        alphabeticBaseline: 0,
        ideographicBaseline: -values[6],
      });
    }
    __drawText(text, x, y, maxWidth, stroke) {
      text = String(text);
      x = Number(x);
      y = Number(y);
      if (!finite([x, y])) return;
      const metrics = this.measureText(text);
      const direction = resolvedTextDirection(this);
      if (maxWidth === undefined) maxWidth = -1;
      else {
        maxWidth = Number(maxWidth);
        if (!Number.isFinite(maxWidth) || maxWidth <= 0) return;
      }
      const renderedWidth = maxWidth >= 0 ? Math.min(metrics.width, maxWidth) : metrics.width;
      if (this.__textAlign === 'center') x -= renderedWidth / 2;
      else if (
        this.__textAlign === 'right' ||
        (this.__textAlign === 'end' && direction === 'ltr') ||
        (this.__textAlign === 'start' && direction === 'rtl')
      )
        x -= renderedWidth;
      if (this.__textBaseline === 'top') y += metrics.fontBoundingBoxAscent;
      else if (this.__textBaseline === 'hanging') y += metrics.hangingBaseline;
      else if (this.__textBaseline === 'middle')
        y += (metrics.fontBoundingBoxAscent - metrics.fontBoundingBoxDescent) / 2;
      else if (this.__textBaseline === 'ideographic' || this.__textBaseline === 'bottom')
        y -= metrics.fontBoundingBoxDescent;
      const style = stroke ? this.__strokeStyle : this.__fillStyle;
      call(
        'canvas2dDrawText',
        this.canvas,
        this.canvas.width,
        this.canvas.height,
        text,
        x,
        y,
        fontSize(this.__font),
        fontFamily(this.__font),
        maxWidth,
        stroke,
        strokePayload(this),
        stylePayload(style, this.__globalAlpha),
        ...this.__transform,
        effectsPayload(this),
        this.__globalCompositeOperation,
        direction,
      );
    }
    fillText(text, x, y, maxWidth = undefined) {
      this.__drawText(text, x, y, maxWidth, false);
    }
    strokeText(text, x, y, maxWidth = undefined) {
      this.__drawText(text, x, y, maxWidth, true);
    }
    beginPath() {
      call('canvas2dBeginPath', this.canvas, this.canvas.width, this.canvas.height);
    }
    closePath() {
      call('canvas2dClosePath', this.canvas, this.canvas.width, this.canvas.height);
    }
    moveTo(x, y) {
      call(
        'canvas2dPathPoint',
        this.canvas,
        this.canvas.width,
        this.canvas.height,
        'move',
        Number(x),
        Number(y),
        ...this.__transform,
      );
    }
    lineTo(x, y) {
      call(
        'canvas2dPathPoint',
        this.canvas,
        this.canvas.width,
        this.canvas.height,
        'line',
        Number(x),
        Number(y),
        ...this.__transform,
      );
    }
    quadraticCurveTo(controlX, controlY, x, y) {
      call(
        'canvas2dPathPoint',
        this.canvas,
        this.canvas.width,
        this.canvas.height,
        'quadratic',
        Number(controlX),
        Number(controlY),
        Number(x),
        Number(y),
        ...this.__transform,
      );
    }
    bezierCurveTo(firstX, firstY, secondX, secondY, x, y) {
      call(
        'canvas2dPathPoint',
        this.canvas,
        this.canvas.width,
        this.canvas.height,
        'bezier',
        Number(firstX),
        Number(firstY),
        Number(secondX),
        Number(secondY),
        Number(x),
        Number(y),
        ...this.__transform,
      );
    }
    arcTo(firstX, firstY, secondX, secondY, radius) {
      [firstX, firstY, secondX, secondY, radius] = [firstX, firstY, secondX, secondY, radius].map(
        Number,
      );
      if (radius < 0) throw new DOMException('The radius provided is negative', 'IndexSizeError');
      if (!finite([firstX, firstY, secondX, secondY, radius])) return;
      call(
        'canvas2dPathArcTo',
        this.canvas,
        this.canvas.width,
        this.canvas.height,
        firstX,
        firstY,
        secondX,
        secondY,
        radius,
        ...this.__transform,
      );
    }
    rect(x, y, width, height) {
      call(
        'canvas2dPathRect',
        this.canvas,
        this.canvas.width,
        this.canvas.height,
        Number(x),
        Number(y),
        Number(width),
        Number(height),
        ...this.__transform,
      );
    }
    roundRect(x, y, width, height, radii = 0) {
      [x, y, width, height] = [x, y, width, height].map(Number);
      if (!finite([x, y, width, height])) return;
      const corners = roundRectRadii(radii, width, height);
      if (width < 0) {
        x += width;
        width = -width;
      }
      if (height < 0) {
        y += height;
        height = -height;
      }
      call(
        'canvas2dPathRoundRect',
        this.canvas,
        this.canvas.width,
        this.canvas.height,
        x,
        y,
        width,
        height,
        ...corners,
        ...this.__transform,
      );
    }
    arc(x, y, radius, startAngle, endAngle, counterclockwise = false) {
      [x, y, radius, startAngle, endAngle] = [x, y, radius, startAngle, endAngle].map(Number);
      if (radius < 0) throw new DOMException('The radius provided is negative', 'IndexSizeError');
      if (!finite([x, y, radius, startAngle, endAngle])) return;
      const sweep = normalizedSweep(startAngle, endAngle, Boolean(counterclockwise));
      call(
        'canvas2dPathArc',
        this.canvas,
        this.canvas.width,
        this.canvas.height,
        x,
        y,
        radius,
        startAngle,
        sweep,
        ...this.__transform,
      );
    }
    ellipse(x, y, radiusX, radiusY, rotation, startAngle, endAngle, counterclockwise = false) {
      [x, y, radiusX, radiusY, rotation, startAngle, endAngle] = [
        x,
        y,
        radiusX,
        radiusY,
        rotation,
        startAngle,
        endAngle,
      ].map(Number);
      if (radiusX < 0 || radiusY < 0)
        throw new DOMException('The radius provided is negative', 'IndexSizeError');
      if (!finite([x, y, radiusX, radiusY, rotation, startAngle, endAngle])) return;
      const sweep = normalizedSweep(startAngle, endAngle, Boolean(counterclockwise));
      call(
        'canvas2dPathEllipse',
        this.canvas,
        this.canvas.width,
        this.canvas.height,
        x,
        y,
        radiusX,
        radiusY,
        rotation,
        startAngle,
        sweep,
        ...this.__transform,
      );
    }
    fill(pathOrRule = 'nonzero', fillRule = 'nonzero') {
      const path = pathOrRule instanceof Path2D ? pathOrRule.__id : 0;
      if (path === 0) fillRule = pathOrRule;
      fillRule = String(fillRule);
      if (fillRule !== 'nonzero' && fillRule !== 'evenodd') return;
      call(
        'canvas2dDrawPath',
        this.canvas,
        this.canvas.width,
        this.canvas.height,
        'fill',
        fillRule,
        stylePayload(this.__fillStyle, this.__globalAlpha),
        strokePayload(this),
        effectsPayload(this),
        this.__globalCompositeOperation,
        path,
      );
    }
    stroke(path = undefined) {
      if (path !== undefined && !(path instanceof Path2D))
        throw new TypeError('stroke expects a Path2D');
      call(
        'canvas2dDrawPath',
        this.canvas,
        this.canvas.width,
        this.canvas.height,
        'stroke',
        'nonzero',
        stylePayload(this.__strokeStyle, this.__globalAlpha),
        strokePayload(this),
        effectsPayload(this),
        this.__globalCompositeOperation,
        path?.__id ?? 0,
      );
    }
    clip(pathOrRule = 'nonzero', fillRule = 'nonzero') {
      const path = pathOrRule instanceof Path2D ? pathOrRule.__id : 0;
      if (path === 0) fillRule = pathOrRule;
      fillRule = String(fillRule);
      if (fillRule !== 'nonzero' && fillRule !== 'evenodd') return;
      call('canvas2dClip', this.canvas, this.canvas.width, this.canvas.height, fillRule, path);
    }
    isPointInPath(pathOrX, xOrY, yOrRule = 'nonzero', maybeRule = 'nonzero') {
      const path = pathOrX instanceof Path2D ? pathOrX.__id : 0;
      const x = path === 0 ? pathOrX : xOrY;
      const y = path === 0 ? xOrY : yOrRule;
      let fillRule = path === 0 ? yOrRule : maybeRule;
      fillRule = String(fillRule);
      if (fillRule !== 'nonzero' && fillRule !== 'evenodd') return false;
      return call(
        'canvas2dIsPointInPath',
        this.canvas,
        this.canvas.width,
        this.canvas.height,
        Number(x),
        Number(y),
        fillRule,
        path,
      );
    }
    isPointInStroke(pathOrX, xOrY, maybeY = undefined) {
      const path = pathOrX instanceof Path2D ? pathOrX.__id : 0;
      const x = path === 0 ? pathOrX : xOrY;
      const y = path === 0 ? xOrY : maybeY;
      return call(
        'canvas2dIsPointInStroke',
        this.canvas,
        this.canvas.width,
        this.canvas.height,
        Number(x),
        Number(y),
        strokePayload(this),
        path,
      );
    }
    createImageData(widthOrData, heightOrSettings = undefined, settings = undefined) {
      if (widthOrData instanceof ImageData) {
        return new ImageData(widthOrData.width, widthOrData.height, {
          colorSpace: widthOrData.colorSpace,
        });
      }
      let width = Math.trunc(Number(widthOrData));
      let height = Math.trunc(Number(heightOrSettings));
      if (width === 0 || height === 0)
        throw new DOMException('The source width or height is zero', 'IndexSizeError');
      width = Math.abs(width);
      height = Math.abs(height);
      const normalized = normalizeImageDataSettings(settings, this.__attributes.colorSpace);
      return new ImageData(width, height, normalized);
    }
    getImageData(sourceX, sourceY, sourceWidth, sourceHeight, settings = undefined) {
      sourceX = Math.trunc(Number(sourceX));
      sourceY = Math.trunc(Number(sourceY));
      sourceWidth = Math.trunc(Number(sourceWidth));
      sourceHeight = Math.trunc(Number(sourceHeight));
      if (
        !finite([sourceX, sourceY, sourceWidth, sourceHeight]) ||
        sourceWidth === 0 ||
        sourceHeight === 0
      ) {
        throw new DOMException('The source width or height is zero', 'IndexSizeError');
      }
      if (sourceWidth < 0) {
        sourceX += sourceWidth;
        sourceWidth = -sourceWidth;
      }
      if (sourceHeight < 0) {
        sourceY += sourceHeight;
        sourceHeight = -sourceHeight;
      }
      ensureOriginClean(this.canvas);
      const normalized = normalizeImageDataSettings(settings, this.__attributes.colorSpace);
      const bytes = call(
        'canvas2dGetImageData',
        this.canvas,
        this.canvas.width,
        this.canvas.height,
        sourceX,
        sourceY,
        sourceWidth,
        sourceHeight,
        normalized.colorSpace,
        normalized.pixelFormat,
      );
      const data =
        normalized.pixelFormat === 'rgba-float16'
          ? new Float16Array(bytes.buffer, bytes.byteOffset, bytes.byteLength / 2)
          : bytes;
      return new ImageData(data, sourceWidth, sourceHeight, normalized);
    }
    putImageData(
      imageData,
      destinationX,
      destinationY,
      dirtyX = undefined,
      dirtyY = undefined,
      dirtyWidth = undefined,
      dirtyHeight = undefined,
    ) {
      if (!(imageData instanceof ImageData))
        throw new TypeError('The first argument must be an ImageData');
      destinationX = Math.trunc(Number(destinationX));
      destinationY = Math.trunc(Number(destinationY));
      if (!finite([destinationX, destinationY])) return;
      if (dirtyX === undefined) {
        call(
          'canvas2dPutImageData',
          this.canvas,
          this.canvas.width,
          this.canvas.height,
          destinationX,
          destinationY,
          imageData.width,
          imageData.height,
          imageData.data,
          imageData.colorSpace,
          imageData.pixelFormat,
        );
        return;
      }
      dirtyX = Math.trunc(Number(dirtyX));
      dirtyY = Math.trunc(Number(dirtyY));
      dirtyWidth = Math.trunc(Number(dirtyWidth));
      dirtyHeight = Math.trunc(Number(dirtyHeight));
      if (!finite([dirtyX, dirtyY, dirtyWidth, dirtyHeight])) return;
      if (dirtyWidth < 0) {
        dirtyX += dirtyWidth;
        dirtyWidth = -dirtyWidth;
      }
      if (dirtyHeight < 0) {
        dirtyY += dirtyHeight;
        dirtyHeight = -dirtyHeight;
      }
      const left = Math.max(0, dirtyX);
      const top = Math.max(0, dirtyY);
      const right = Math.min(imageData.width, dirtyX + dirtyWidth);
      const bottom = Math.min(imageData.height, dirtyY + dirtyHeight);
      if (right <= left || bottom <= top) return;
      const clippedWidth = right - left;
      const clippedHeight = bottom - top;
      const pixels = new imageData.data.constructor(clippedWidth * clippedHeight * 4);
      for (let row = 0; row < clippedHeight; row++) {
        const start = ((top + row) * imageData.width + left) * 4;
        pixels.set(
          imageData.data.subarray(start, start + clippedWidth * 4),
          row * clippedWidth * 4,
        );
      }
      call(
        'canvas2dPutImageData',
        this.canvas,
        this.canvas.width,
        this.canvas.height,
        destinationX + left,
        destinationY + top,
        clippedWidth,
        clippedHeight,
        pixels,
        imageData.colorSpace,
        imageData.pixelFormat,
      );
    }
  }

  const widthDescriptor = Object.getOwnPropertyDescriptor(HTMLCanvasElement.prototype, 'width');
  const heightDescriptor = Object.getOwnPropertyDescriptor(HTMLCanvasElement.prototype, 'height');
  const resetBitmap = (canvas) => {
    call('canvasReset', canvas, canvas.width, canvas.height);
    contexts.get(canvas)?.context?.__resetState?.();
  };
  Object.defineProperty(HTMLCanvasElement.prototype, 'width', {
    get: widthDescriptor.get,
    set(value) {
      widthDescriptor.set.call(this, value);
      resetBitmap(this);
    },
    enumerable: widthDescriptor.enumerable,
    configurable: true,
  });
  Object.defineProperty(HTMLCanvasElement.prototype, 'height', {
    get: heightDescriptor.get,
    set(value) {
      heightDescriptor.set.call(this, value);
      resetBitmap(this);
    },
    enumerable: heightDescriptor.enumerable,
    configurable: true,
  });

  canvas.register('2d', function (options) {
    options = options !== null && typeof options === 'object' ? options : {};
    const colorSpace = options.colorSpace === undefined ? 'srgb' : String(options.colorSpace);
    const colorType = options.colorType === undefined ? 'unorm8' : String(options.colorType);
    if (colorSpace !== 'srgb' && colorSpace !== 'display-p3')
      throw new TypeError('Invalid Canvas colorSpace');
    if (colorType !== 'unorm8' && colorType !== 'float16')
      throw new TypeError('Invalid Canvas colorType');
    const attributes = {
      alpha: options.alpha === undefined ? true : Boolean(options.alpha),
      desynchronized: Boolean(options.desynchronized),
      colorSpace,
      colorType,
      willReadFrequently: Boolean(options.willReadFrequently),
    };
    if (
      !call(
        'canvas2dAcquire',
        this,
        this.width,
        this.height,
        attributes.alpha,
        colorSpace,
        colorType,
      )
    )
      return null;
    const context = new CanvasRenderingContext2D(construct, this, attributes);
    contexts.set(this, { type: '2d', context });
    return context;
  });

  Object.defineProperties(HTMLCanvasElement.prototype, {
    toDataURL: {
      value(type = 'image/png', quality = undefined) {
        ensureOriginClean(this);
        const normalizedQuality =
          typeof quality === 'number' && quality >= 0 && quality <= 1 ? quality : 0.92;
        return call(
          'canvasEncode',
          this,
          this.width,
          this.height,
          String(type).toLowerCase(),
          Math.round(normalizedQuality * 100),
        );
      },
      writable: true,
      configurable: true,
    },
    toBlob: {
      value(callback, type = 'image/png', quality = undefined) {
        if (typeof callback !== 'function') throw new TypeError('The callback must be a function');
        const dataUrl = this.toDataURL(type, quality);
        setTimeout(() => {
          if (dataUrl === 'data:,') {
            callback(null);
            return;
          }
          const separator = dataUrl.indexOf(',');
          const mimeType = dataUrl.slice(5, dataUrl.indexOf(';', 5));
          const binary = atob(dataUrl.slice(separator + 1));
          const bytes = new Uint8Array(binary.length);
          for (let index = 0; index < binary.length; index++)
            bytes[index] = binary.charCodeAt(index);
          callback(new Blob([bytes], { type: mimeType }));
        }, 0);
      },
      writable: true,
      configurable: true,
    },
  });

  function createImageBitmap(source, ...arguments_) {
    return Promise.resolve().then(() => {
      let crop = null;
      let options;
      if (arguments_.length === 0) {
        options = {};
      } else if (arguments_.length === 1) {
        options = arguments_[0] ?? {};
      } else if (arguments_.length === 4 || arguments_.length === 5) {
        crop = arguments_.slice(0, 4).map((value) => Math.trunc(Number(value)));
        options = arguments_[4] ?? {};
      } else {
        throw new TypeError('createImageBitmap requires 1, 2, 5, or 6 arguments');
      }
      if (typeof options !== 'object') throw new TypeError('ImageBitmapOptions must be an object');

      let drawable = source;
      let temporaryBitmap = null;
      let width;
      let height;
      if (source instanceof Blob) {
        const metadata = JSON.parse(call('canvasDecodeImageBitmap', globalThis, source.__bytes));
        drawable = temporaryBitmap = new ImageBitmap(construct, metadata);
        width = drawable.width;
        height = drawable.height;
      } else if (source instanceof HTMLCanvasElement || source instanceof ImageBitmap) {
        if (source instanceof ImageBitmap && source.__id === 0)
          throw new DOMException('The ImageBitmap is closed', 'InvalidStateError');
        width = source.width;
        height = source.height;
      } else if (source instanceof HTMLImageElement) {
        const metadata = JSON.parse(call('imageMetadata', source));
        if (!metadata.complete || metadata.width === 0 || metadata.height === 0)
          throw new DOMException('The image has no decoded image data', 'InvalidStateError');
        width = metadata.width;
        height = metadata.height;
      } else if (source instanceof ImageData) {
        width = source.width;
        height = source.height;
        const imageDataCanvas = document.createElement('canvas');
        imageDataCanvas.width = width;
        imageDataCanvas.height = height;
        imageDataCanvas.getContext('2d').putImageData(source, 0, 0);
        drawable = imageDataCanvas;
      } else {
        throw new TypeError('Unsupported ImageBitmapSource');
      }

      let [sourceX, sourceY, sourceWidth, sourceHeight] = crop ?? [0, 0, width, height];
      if (![sourceX, sourceY, sourceWidth, sourceHeight].every(Number.isFinite))
        throw new RangeError('ImageBitmap crop rectangle must be finite');
      if (sourceWidth === 0 || sourceHeight === 0)
        throw new RangeError('ImageBitmap crop dimensions must be non-zero');
      if (sourceWidth < 0) {
        sourceX += sourceWidth;
        sourceWidth = -sourceWidth;
      }
      if (sourceHeight < 0) {
        sourceY += sourceHeight;
        sourceHeight = -sourceHeight;
      }
      const targetWidth =
        options.resizeWidth === undefined ? sourceWidth : Number(options.resizeWidth) >>> 0;
      const targetHeight =
        options.resizeHeight === undefined ? sourceHeight : Number(options.resizeHeight) >>> 0;
      if (targetWidth === 0 || targetHeight === 0)
        throw new RangeError('ImageBitmap resize dimensions must be non-zero');

      const target = document.createElement('canvas');
      target.width = targetWidth;
      target.height = targetHeight;
      const context = target.getContext('2d');
      context.imageSmoothingQuality = String(options.resizeQuality ?? 'low');
      if (options.imageOrientation === 'flipY') {
        context.translate(0, targetHeight);
        context.scale(1, -1);
      }
      context.drawImage(
        drawable,
        sourceX,
        sourceY,
        sourceWidth,
        sourceHeight,
        0,
        0,
        targetWidth,
        targetHeight,
      );
      temporaryBitmap?.close();
      return new ImageBitmap(
        construct,
        JSON.parse(call('canvasCreateImageBitmap', target, target.width, target.height)),
      );
    });
  }

  Object.defineProperties(globalThis, {
    CanvasRenderingContext2D: {
      value: CanvasRenderingContext2D,
      writable: true,
      configurable: true,
    },
    ImageBitmap: { value: ImageBitmap, writable: true, configurable: true },
    createImageBitmap: { value: createImageBitmap, writable: true, configurable: true },
    CanvasGradient: { value: CanvasGradient, writable: true, configurable: true },
    CanvasPattern: { value: CanvasPattern, writable: true, configurable: true },
    Path2D: { value: Path2D, writable: true, configurable: true },
    TextMetrics: { value: TextMetrics, writable: true, configurable: true },
  });

  // Legacy callable placeholders; no capture or offscreen rendering ownership.
  for (const name of ['captureStream', 'transferControlToOffscreen']) {
    Object.defineProperty(HTMLCanvasElement.prototype, name, {
      value: { [name]() {} }[name],
      writable: true,
      configurable: true,
    });
  }
}
