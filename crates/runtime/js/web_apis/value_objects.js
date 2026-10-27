(() => {
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

  const float16ImageData = (value) =>
    typeof Float16Array !== 'undefined' && value instanceof Float16Array;

  class ImageData {
    constructor(dataOrWidth, widthOrHeight, heightOrSettings = undefined) {
      let data;
      let width;
      let height;
      let settings;
      let pixelFormat;
      if (dataOrWidth instanceof Uint8ClampedArray || float16ImageData(dataOrWidth)) {
        data = dataOrWidth;
        pixelFormat = float16ImageData(data) ? 'rgba-float16' : 'rgba-unorm8';
        width = Number(widthOrHeight) >>> 0;
        if (heightOrSettings !== undefined && typeof heightOrSettings !== 'object') {
          height = Number(heightOrSettings) >>> 0;
          settings = arguments[3];
        } else {
          if (width === 0 || data.length % (width * 4) !== 0) {
            throw new DOMException(
              'The input data length is not a multiple of (4 * width)',
              'IndexSizeError',
            );
          }
          height = data.length / (width * 4);
          settings = heightOrSettings;
        }
        if (width === 0 || height === 0 || data.length !== width * height * 4) {
          throw new DOMException(
            'The input data length does not match the dimensions',
            'IndexSizeError',
          );
        }
        const normalized = normalizeImageDataSettings(settings, 'srgb', pixelFormat);
        if (normalized.pixelFormat !== pixelFormat) {
          throw new DOMException(
            'The data and settings pixel formats do not match',
            'InvalidStateError',
          );
        }
        settings = normalized;
      } else {
        width = Number(dataOrWidth) >>> 0;
        height = Number(widthOrHeight) >>> 0;
        if (width === 0 || height === 0) {
          throw new DOMException('The source width or height is zero', 'IndexSizeError');
        }
        settings = normalizeImageDataSettings(heightOrSettings);
        pixelFormat = settings.pixelFormat;
        data =
          pixelFormat === 'rgba-float16'
            ? new Float16Array(width * height * 4)
            : new Uint8ClampedArray(width * height * 4);
      }
      Object.defineProperties(this, {
        data: { value: data, enumerable: true },
        width: { value: width, enumerable: true },
        height: { value: height, enumerable: true },
        colorSpace: { value: settings.colorSpace, enumerable: true },
        pixelFormat: { value: pixelFormat, enumerable: true },
      });
    }
  }

  Object.assign(globalThis, { ImageData });
})();
