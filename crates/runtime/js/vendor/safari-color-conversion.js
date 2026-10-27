import { HSL_to_sRGB, HWB_to_sRGB, sRGB_to_HSL, sRGB_to_Hue } from '@csstools/color-helpers';

// WebKit converts these members of the sRGB family directly. An XYZ round trip
// changes relative-color channels and can move a value across a byte boundary.
export function directSRGBConversion(color, destination) {
  const family = ['rgb', 'hex', 'srgb', 'hsl', 'hwb'];
  if (!family.includes(color.colorNotation) || !family.includes(destination)) return null;
  const channels = color.channels.map((value) => (Number.isNaN(value) ? 0 : value));
  const rgb =
    color.colorNotation === 'hsl'
      ? HSL_to_sRGB(channels)
      : color.colorNotation === 'hwb'
        ? HWB_to_sRGB(channels)
        : channels;
  if (destination === 'hsl') return sRGB_to_HSL(rgb);
  if (destination === 'hwb') {
    const white = Math.min(...rgb),
      black = 1 - Math.max(...rgb);
    return [white + black >= 0.99999 ? NaN : sRGB_to_Hue(rgb), white * 100, black * 100];
  }
  return rgb;
}
