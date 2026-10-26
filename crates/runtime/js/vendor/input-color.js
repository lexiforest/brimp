import {color} from '@csstools/css-color-parser';
import {parseListOfComponentValues, isWhiteSpaceOrCommentNode} from '@csstools/css-parser-algorithms';
import {tokenize} from '@csstools/css-tokenizer';
import {
    a98_RGB_to_XYZ_D65, HSL_to_XYZ_D65, HWB_to_XYZ_D65,
    Lab_to_XYZ_D65, LCH_to_XYZ_D65, OKLab_to_XYZ_D65, OKLCH_to_XYZ_D65,
    P3_to_XYZ_D65, lin_P3_to_XYZ_D65, lin_sRGB_to_XYZ_D65,
    ProPhoto_RGB_to_XYZ_D65, rec_2020_to_XYZ_D65,
    XYZ_D50_to_XYZ_D65, XYZ_D65_to_sRGB, sRGB_to_XYZ_D65, XYZ_D65_to_P3,
    HSL_to_sRGB, HWB_to_sRGB,
} from '@csstools/color-helpers';

const toXYZ = {
    'a98-rgb': a98_RGB_to_XYZ_D65, hsl: HSL_to_XYZ_D65, hwb: HWB_to_XYZ_D65,
    lab: Lab_to_XYZ_D65, lch: LCH_to_XYZ_D65, oklab: OKLab_to_XYZ_D65, oklch: OKLCH_to_XYZ_D65,
    'display-p3': P3_to_XYZ_D65, 'display-p3-linear': lin_P3_to_XYZ_D65,
    'srgb-linear': lin_sRGB_to_XYZ_D65, 'prophoto-rgb': ProPhoto_RGB_to_XYZ_D65,
    rec2020: rec_2020_to_XYZ_D65, 'xyz-d50': XYZ_D50_to_XYZ_D65, 'xyz-d65': value => value,
};

// Installed Safari 27.0 light appearance defaults. Input color parsing resolves
// currentColor to black, independently of the element's computed text color.
// These are runtime defaults, not a live connection to the OS theme.
const systemColors = {
    currentcolor: '#000000', accentcolor: '#007aff', accentcolortext: '#000000',
    activetext: '#ff0000', buttonborder: '#ffffff', buttonface: '#c0c0c0',
    buttontext: '#000000d8', canvas: '#ffffff', canvastext: '#000000',
    field: '#ffffff', fieldtext: '#000000d8', graytext: '#0000003f',
    highlight: '#80bcfe99', highlighttext: '#000000', linktext: '#0000ee',
    mark: '#ffff00', marktext: '#000000', selecteditem: '#c0c0c0',
    selecteditemtext: '#000000', visitedtext: '#551a8b',
};
const byte = value => {
    const scaled = Math.fround(Math.fround(value) * 255);
    // Safari's native long conversion resolves non-finite and overflowing
    // extended channels to zero before clamping to a byte.
    return !Number.isFinite(scaled) || scaled >= 2 ** 63 ? 0 : Math.max(0, Math.min(255, Math.round(scaled)));
};
const decimal = value => String(Number((value / 255).toFixed(6)));
const component = value => {
    value = Math.fround(value);
    if (Number.isNaN(value)) return '0';
    if (!Number.isFinite(value)) return value < 0 ? 'calc(-infinity)' : 'calc(infinity)';
    if (Number.isInteger(value)) return BigInt(value).toString();
    return String(Number(value.toFixed(6)));
};

export function sanitize(value, alpha, displayP3 = false) {
    // Replace complete identifier tokens, including escaped identifiers, without
    // changing comments, function names, channel names or arbitrary substrings.
    const tokens = tokenize({css: value}).flatMap(token => {
        const name = token[0] === 'ident-token' ? token[4].value.toLowerCase() : '';
        const replacement = Object.hasOwn(systemColors, name) && systemColors[name];
        return replacement ? tokenize({css: replacement}).slice(0, -1) : [token];
    });
    const nodes = parseListOfComponentValues(tokens).filter(node => !isWhiteSpaceOrCommentNode(node));
    const node = nodes.length === 1 && nodes[0];
    const parsed = node && color(node);
    let rgba = [0, 0, 0, 255];
    let p3 = [0, 0, 0], opacity = 1;
    if (parsed && typeof parsed.alpha === 'number') {
        const channels = parsed.channels.map(value => Number.isNaN(value) ? 0 : Math.fround(value));
        const convert = toXYZ[parsed.colorNotation];
        const rgb = parsed.colorNotation === 'hsl' ? HSL_to_sRGB(channels) :
            parsed.colorNotation === 'hwb' ? HWB_to_sRGB(channels) :
            convert ? XYZ_D65_to_sRGB(convert(channels)) : channels;
        const byteColor = ['rgb', 'hex', 'hsl', 'hwb'].includes(parsed.colorNotation) &&
            !parsed.syntaxFlags.has('relative-color-syntax') && !parsed.syntaxFlags.has('color-mix');
        rgba = [...rgb.map(value => byte(byteColor ? Math.max(0, Math.min(1, value)) : value)),
            byte(Number.isNaN(parsed.alpha) ? 0 : parsed.alpha)];
        if (displayP3) {
            opacity = Number.isNaN(parsed.alpha) ? 0 : Math.fround(parsed.alpha);
            if (byteColor) {
                p3 = XYZ_D65_to_P3(sRGB_to_XYZ_D65(rgba.slice(0,3).map(value => value / 255)));
                opacity = rgba[3] / 255;
            } else if (parsed.colorNotation === 'display-p3') p3 = channels;
            else p3 = XYZ_D65_to_P3(['hsl','hwb'].includes(parsed.colorNotation) ? sRGB_to_XYZ_D65(rgb) :
                convert ? convert(channels) : sRGB_to_XYZ_D65(channels));
        }
    }
    if (displayP3) return 'color(display-p3 ' + p3.map(component).join(' ') +
        (!alpha || opacity === 1 ? '' : ' / ' + component(opacity)) + ')';
    if (!alpha) return '#' + rgba.slice(0, 3).map(value => value.toString(16).padStart(2, '0')).join('');
    return 'color(srgb ' + rgba.slice(0, 3).map(decimal).join(' ') +
        (rgba[3] === 255 ? '' : ' / ' + decimal(rgba[3])) + ')';
}
