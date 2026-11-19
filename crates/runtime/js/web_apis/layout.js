// ---- Layout ---------------------------------------------------------------
// Box layout for element geometry and computed style. Text runs are measured
// through the platform font manager, so a page's metrics follow the fonts the
// profile claims instead of a guessed advance table.
//
// Scope: normal flow for block and inline content, margins/padding/borders,
// explicit sizes, and relative/absolute positioning. There are no floats, no
// flex/grid containers, and one box per element: an inline element reports the
// bounding box of its line fragments rather than one rect per fragment.

const __layoutMachine = { profile: null, defaultFamily: '-apple-system' };
const __layoutState = { version: 0, boxes: null, fragments: null, observer: null };
const __layoutTextCache = new Map();
const __layoutComputedCache = new WeakMap();
const __layoutRulesCache = new WeakMap();

function __layoutSetProfile(profile) {
  __layoutMachine.profile = profile;
  const family = profile?.fonts?.default_family;
  if (typeof family === 'string' && family) __layoutMachine.defaultFamily = family;
  __layoutInvalidate();
}

// Called from the host bridge for every DOM mutation that can change a box.
function __layoutMutationBefore() {
  __layoutState.version++;
  __layoutState.boxes = null;
  __layoutState.fragments = null;
}

function __layoutInvalidate() {
  __layoutState.version++;
  __layoutState.boxes = null;
  __layoutState.fragments = null;
}

function __layoutViewport() {
  return {
    width: Number(globalThis.innerWidth) || 0,
    height: Number(globalThis.innerHeight) || 0,
  };
}

// A length resolves against the element's own font size; percentages need the
// base the property defines, which the caller supplies.
function __layoutLength(value, context = {}) {
  if (value == null) return null;
  const text = String(value).trim().toLowerCase();
  if (text === '' || text === 'auto' || text === 'none' || text === 'normal') return null;
  const fontSize = context.fontSize ?? 16;
  const rootFontSize = context.rootFontSize ?? 16;
  const percentBase = context.percentBase;
  const match = /^(-?[0-9.]+)([a-z%]*)$/.exec(text);
  if (!match) return null;
  const number = Number.parseFloat(match[1]);
  if (!Number.isFinite(number)) return null;
  switch (match[2]) {
    case '':
      return context.unitless ? number : null;
    case 'px':
      return number;
    case 'em':
      return number * fontSize;
    case 'rem':
      return number * rootFontSize;
    case 'ex':
    case 'ch':
      return number * fontSize * 0.5;
    case 'pt':
      return (number * 4) / 3;
    case 'pc':
      return number * 16;
    case 'in':
      return number * 96;
    case 'cm':
      return (number * 96) / 2.54;
    case 'mm':
      return (number * 96) / 25.4;
    case 'q':
      return (number * 96) / 101.6;
    case 'vw':
      return (number * (context.viewportWidth ?? 0)) / 100;
    case 'vh':
      return (number * (context.viewportHeight ?? 0)) / 100;
    case '%':
      return percentBase == null ? null : (number * percentBase) / 100;
    default:
      return null;
  }
}

const __layoutFontSizes = {
  'xx-small': 9,
  'x-small': 10,
  small: 13,
  medium: 16,
  large: 18,
  'x-large': 24,
  'xx-large': 32,
  'xxx-large': 48,
};

function __layoutFontSize(value, context) {
  const text = String(value ?? '')
    .trim()
    .toLowerCase();
  if (text === '') return null;
  if (Object.prototype.hasOwnProperty.call(__layoutFontSizes, text)) return __layoutFontSizes[text];
  if (text === 'larger') return (context.fontSize ?? 16) * 1.2;
  if (text === 'smaller') return (context.fontSize ?? 16) / 1.2;
  return __layoutLength(text, context);
}

const __layoutBlockTags = new Set([
  'html',
  'body',
  'div',
  'p',
  'h1',
  'h2',
  'h3',
  'h4',
  'h5',
  'h6',
  'ul',
  'ol',
  'li',
  'dl',
  'dt',
  'dd',
  'blockquote',
  'pre',
  'form',
  'fieldset',
  'legend',
  'figure',
  'figcaption',
  'header',
  'footer',
  'section',
  'nav',
  'article',
  'aside',
  'main',
  'address',
  'hr',
  'table',
  'thead',
  'tbody',
  'tfoot',
  'tr',
  'td',
  'th',
  'caption',
  'details',
  'summary',
  'dialog',
  'menu',
  'noscript',
  'center',
  'output',
]);
const __layoutHiddenTags = new Set([
  'head',
  'script',
  'style',
  'template',
  'link',
  'meta',
  'title',
  'base',
  'area',
  'datalist',
  'param',
  'source',
  'track',
  'col',
  'colgroup',
  'rp',
]);
const __layoutInlineBlockTags = new Set([
  'button',
  'input',
  'select',
  'textarea',
  'canvas',
  'video',
  'audio',
  'iframe',
  'object',
  'embed',
  'img',
  'progress',
  'meter',
  'svg',
  'math',
]);
const __layoutMonoTags = new Set(['code', 'kbd', 'samp', 'var', 'pre', 'tt']);

// User-agent defaults for the properties layout consumes.
function __layoutUserAgent(tag) {
  const style = {
    display: __layoutHiddenTags.has(tag)
      ? 'none'
      : __layoutBlockTags.has(tag)
        ? 'block'
        : __layoutInlineBlockTags.has(tag)
          ? 'inline-block'
          : 'inline',
    position: 'static',
    // Font properties are inherited, so the user agent leaves them unset and
    // only the elements that define them (headings, `code`, `b`) declare one.
    'box-sizing': 'content-box',
    'margin-top': '0px',
    'margin-right': '0px',
    'margin-bottom': '0px',
    'margin-left': '0px',
    'padding-top': '0px',
    'padding-right': '0px',
    'padding-bottom': '0px',
    'padding-left': '0px',
    'border-top-width': '0px',
    'border-right-width': '0px',
    'border-bottom-width': '0px',
    'border-left-width': '0px',
    'border-top-style': 'none',
    'border-right-style': 'none',
    'border-bottom-style': 'none',
    'border-left-style': 'none',
    'text-align': 'start',
    'white-space': 'normal',
    visibility: 'visible',
    overflow: 'visible',
    width: 'auto',
    height: 'auto',
  };
  const em = (n) => `${n}em`;
  switch (tag) {
    case 'body':
      // The user-agent block margin is a shorthand; expand it so the resolved
      // per-side values stay the single source layout reads.
      style['margin-top'] = '8px';
      style['margin-right'] = '8px';
      style['margin-bottom'] = '8px';
      style['margin-left'] = '8px';
      break;
    case 'p':
    case 'dl':
      style['margin-top'] = em(1);
      style['margin-bottom'] = em(1);
      break;
    case 'blockquote':
    case 'figure':
    case 'menu':
      style['margin-top'] = em(1);
      style['margin-bottom'] = em(1);
      style['margin-left'] = '40px';
      style['margin-right'] = '40px';
      break;
    case 'pre':
      style['margin-top'] = em(1);
      style['margin-bottom'] = em(1);
      break;
    case 'h1':
      style['font-size'] = em(2);
      style['font-weight'] = '700';
      style['margin-top'] = '0.67em';
      style['margin-bottom'] = '0.67em';
      break;
    case 'h2':
      style['font-size'] = em(1.5);
      style['font-weight'] = '700';
      style['margin-top'] = '0.83em';
      style['margin-bottom'] = '0.83em';
      break;
    case 'h3':
      style['font-size'] = em(1.17);
      style['font-weight'] = '700';
      style['margin-top'] = em(1);
      style['margin-bottom'] = em(1);
      break;
    case 'h4':
      style['font-weight'] = '700';
      style['margin-top'] = '1.33em';
      style['margin-bottom'] = '1.33em';
      break;
    case 'h5':
      style['font-size'] = em(0.83);
      style['font-weight'] = '700';
      style['margin-top'] = '1.67em';
      style['margin-bottom'] = '1.67em';
      break;
    case 'h6':
      style['font-size'] = em(0.67);
      style['font-weight'] = '700';
      style['margin-top'] = '2.33em';
      style['margin-bottom'] = '2.33em';
      break;
    case 'ul':
    case 'ol':
      style['margin-top'] = em(1);
      style['margin-bottom'] = em(1);
      style['padding-left'] = '40px';
      break;
    case 'dd':
      style['margin-left'] = '40px';
      break;
    case 'hr':
      style['margin-top'] = '0.5em';
      style['margin-bottom'] = '0.5em';
      break;
    case 'fieldset':
      style['padding-top'] = '0.35em';
      style['padding-bottom'] = '0.625em';
      style['padding-left'] = '0.75em';
      style['padding-right'] = '0.75em';
      break;
    case 'td':
    case 'th':
      style.display = 'table-cell';
      style['padding-top'] = '1px';
      style['padding-bottom'] = '1px';
      style['padding-left'] = '1px';
      style['padding-right'] = '1px';
      break;
    case 'table':
      style.display = 'table';
      break;
    case 'tr':
      style.display = 'table-row';
      break;
    case 'li':
      style.display = 'list-item';
      break;
  }
  if (tag === 'html') style['font-size'] = '16px';
  if (__layoutMonoTags.has(tag)) {
    style['font-family'] = 'monospace';
    if (tag === 'pre') style['font-size'] = '13px';
  }
  if (tag === 'b' || tag === 'strong' || tag === 'th') style['font-weight'] = '700';
  if (tag === 'i' || tag === 'em' || tag === 'cite' || tag === 'var')
    style['font-style'] = 'italic';
  if (tag === 'small' || tag === 'sup' || tag === 'sub') style['font-size'] = em(0.83);
  if (tag === 'big') style['font-size'] = em(1.17);
  return style;
}

// A declaration block arrives with shorthands intact, so the four-side forms
// are expanded once here rather than in every consumer.
function __layoutExpandShorthand(name, value, into) {
  const text = String(value).trim();
  const parts = text.split(/\s+/).filter((part) => part !== '');
  if (parts.length === 0) return true;
  const box = (prefix, suffix, values) => {
    const [top, right = top, bottom = top, left = right] = values;
    into[`${prefix}top${suffix}`] = top;
    into[`${prefix}right${suffix}`] = right;
    into[`${prefix}bottom${suffix}`] = bottom;
    into[`${prefix}left${suffix}`] = left;
  };
  const isWidth = (part) => /^-?[0-9.]+(px|em|rem|pt|pc|in|cm|mm|q|ex|ch|vw|vh|%)?$/.test(part);
  const isStyle = (part) =>
    [
      'none',
      'hidden',
      'solid',
      'dashed',
      'dotted',
      'double',
      'groove',
      'ridge',
      'inset',
      'outset',
    ].includes(part);
  switch (name) {
    case 'margin':
    case 'padding':
      if (parts.length > 4) return false;
      box(`${name}-`, '', parts);
      return true;
    case 'border-width':
      box('border-', '-width', parts.slice(0, 4));
      return true;
    case 'border-style':
      box('border-', '-style', parts.slice(0, 4));
      return true;
    case 'border-color':
      box('border-', '-color', parts.slice(0, 4));
      return true;
    case 'inset':
      box('', '', parts.slice(0, 4));
      return true;
    case 'border': {
      const width = parts.find((part) => isWidth(part) && !isStyle(part));
      const style = parts.find(isStyle);
      for (const side of ['top', 'right', 'bottom', 'left']) {
        into[`border-${side}-width`] = width ?? 'medium';
        into[`border-${side}-style`] = style ?? 'none';
      }
      return true;
    }
    case 'border-top':
    case 'border-right':
    case 'border-bottom':
    case 'border-left': {
      const side = name.slice('border-'.length);
      into[`border-${side}-width`] =
        parts.find((part) => isWidth(part) && !isStyle(part)) ?? 'medium';
      into[`border-${side}-style`] = parts.find(isStyle) ?? 'none';
      return true;
    }
    case 'font': {
      // [style] [variant] [weight] size[/line-height] family-list
      const size =
        /(-?[0-9.]+(?:px|em|rem|%|pt|pc|in|cm|mm|q|ex|ch|vw|vh)|xx-small|x-small|small|medium|large|x-large|xx-large|xxx-large|larger|smaller)(?:\s*\/\s*([^\s]+))?/i.exec(
          text,
        );
      if (!size) return false;
      into['font-size'] = size[1];
      if (size[2]) into['line-height'] = size[2];
      const family = text.slice(size.index + size[0].length).trim();
      if (family) into['font-family'] = family;
      if (/\bitalic\b|\boblique\b/i.test(text)) into['font-style'] = 'italic';
      const bold = /\bbold\b|\bbolder\b/i.test(text);
      const numeric = /\b([1-9]00)\b/.exec(text);
      if (bold) into['font-weight'] = '700';
      else if (numeric) into['font-weight'] = numeric[1];
      return true;
    }
    default:
      return false;
  }
}

function __layoutCollectDeclarations(declaration, into) {
  if (!declaration) return;
  const length = Number(declaration.length) || 0;
  for (let index = 0; index < length; index++) {
    const name = declaration.item(index);
    if (!name) continue;
    const value = declaration.getPropertyValue(name);
    if (value.trim() === '') continue;
    if (!__layoutExpandShorthand(name, value, into)) into[name] = value;
  }
}

function __layoutSpecificity(selector) {
  const ids = (selector.match(/#[\w-]+/g) ?? []).length;
  const classes = (selector.match(/\.[\w-]+|\[[^\]]*\]|:(?!:)[\w-]+/g) ?? []).length;
  const stripped = selector
    .replace(/#[\w-]+/g, ' ')
    .replace(/\.[\w-]+/g, ' ')
    .replace(/\[[^\]]*\]/g, ' ')
    .replace(/::?[\w-]+(\([^)]*\))?/g, ' ')
    .replace(/[>+~*,]/g, ' ');
  const elements = stripped.split(/\s+/).filter((part) => /^[a-zA-Z][\w-]*$/.test(part)).length;
  return ids * 10000 + classes * 100 + elements;
}

// A selector list splits on top-level commas only: `:is(a, b)` stays one entry.
function __layoutSelectorList(text) {
  const selectors = [];
  let depth = 0;
  let start = 0;
  for (let index = 0; index < text.length; index++) {
    const character = text[index];
    if (character === '(' || character === '[') depth++;
    else if (character === ')' || character === ']') depth--;
    else if (character === ',' && depth === 0) {
      selectors.push(text.slice(start, index));
      start = index + 1;
    }
  }
  selectors.push(text.slice(start));
  return selectors.map((selector) => selector.trim()).filter((selector) => selector !== '');
}

function __layoutCollectRules(rules, list) {
  if (!rules) return;
  for (const rule of rules) {
    const nested = rule.cssRules;
    if (typeof rule.selectorText === 'string' && rule.style) {
      list.push({ selector: rule.selectorText, style: rule.style, order: list.length });
    } else if (nested) {
      // Conditional groups gate their rules; the media-query engine decides
      // whether this one applies to the profile's viewport.
      const kind = rule.constructor?.name;
      if (kind === 'CSSMediaRule' && typeof rule.conditionText === 'string') {
        let applies = true;
        try {
          const condition = String(rule.conditionText).trim();
          applies = matchMedia(condition.includes('(') ? condition : `(${condition})`).matches;
        } catch {
          applies = true;
        }
        if (!applies) continue;
      }
      __layoutCollectRules(nested, list);
    }
  }
}

function __layoutAuthorRules(document) {
  const cached = __layoutRulesCache.get(document);
  if (cached && cached.version === __layoutState.version) return cached.rules;
  const rules = [];
  try {
    for (const sheet of document.styleSheets) {
      try {
        __layoutCollectRules(sheet.cssRules, rules);
      } catch {
        // Cross-origin or still-loading sheets contribute nothing.
      }
    }
  } catch {
    // No CSSOM on this document.
  }
  __layoutRulesCache.set(document, { version: __layoutState.version, rules });
  return rules;
}

// Declared declarations: user agent, author rules by specificity and source
// order, then the element's own style attribute.
function __layoutDeclared(element) {
  const tag = String(element.localName ?? element.tagName ?? '').toLowerCase();
  const declared = __layoutUserAgent(tag);
  const matches = [];
  for (const rule of __layoutAuthorRules(element.ownerDocument)) {
    let specificity = -1;
    for (const selector of __layoutSelectorList(rule.selector)) {
      try {
        if (element.matches(selector))
          specificity = Math.max(specificity, __layoutSpecificity(selector));
      } catch {
        // A selector this engine cannot compile cannot match.
      }
    }
    if (specificity >= 0) matches.push({ rule, specificity });
  }
  matches.sort((left, right) =>
    left.specificity === right.specificity
      ? left.rule.order - right.rule.order
      : left.specificity - right.specificity,
  );
  for (const { rule } of matches) __layoutCollectDeclarations(rule.style, declared);
  try {
    __layoutCollectDeclarations(element.style, declared);
  } catch {
    // Detached nodes can reject style access; the cascaded value stands.
  }
  return declared;
}

function __layoutIsBold(weight) {
  if (/bold/i.test(String(weight))) return true;
  const numeric = Number.parseInt(weight, 10);
  return Number.isFinite(numeric) && numeric >= 600;
}

// Computed style: declared values resolved to what layout needs.
function __layoutComputed(element) {
  const version = __layoutState.version;
  const cached = __layoutComputedCache.get(element);
  if (cached && cached.version === version) return cached.computed;

  const declared = __layoutDeclared(element);
  const parent = element.parentElement;
  const parentComputed = parent && parent.nodeType === 1 ? __layoutComputed(parent) : null;
  const viewport = __layoutViewport();
  const rootFontSize = 16;
  const inherited = {
    fontSize: parentComputed ? parentComputed.fontSize : 16,
    family: parentComputed ? parentComputed.family : __layoutMachine.defaultFamily,
    weight: parentComputed ? parentComputed.weight : '400',
    fontStyle: parentComputed ? parentComputed.fontStyle : 'normal',
    lineHeight: parentComputed ? parentComputed.lineHeight : null,
    color: parentComputed ? parentComputed.color : 'rgb(0, 0, 0)',
  };
  const fontSize =
    __layoutFontSize(declared['font-size'], {
      fontSize: inherited.fontSize,
      rootFontSize,
      viewportWidth: viewport.width,
      viewportHeight: viewport.height,
    }) ?? inherited.fontSize;
  const context = {
    fontSize,
    rootFontSize,
    viewportWidth: viewport.width,
    viewportHeight: viewport.height,
  };
  const family =
    String(declared['font-family'] ?? '').trim() ||
    inherited.family ||
    __layoutMachine.defaultFamily;
  const lineHeightDeclared =
    declared['line-height'] === undefined ? '' : String(declared['line-height']);
  const weight = String(declared['font-weight'] ?? inherited.weight);
  const fontStyle = String(declared['font-style'] ?? inherited.fontStyle);
  const lineHeightText = lineHeightDeclared.trim().toLowerCase();
  const lineHeight =
    lineHeightText === ''
      ? inherited.lineHeight
      : lineHeightText === 'normal'
        ? null
        : (__layoutLength(lineHeightText, { ...context, unitless: true }) ?? inherited.lineHeight);

  // Percentages wait for layout: they resolve against the containing block,
  // which only the placement pass knows.
  const sides = (prefix) => {
    const resolve = (side) =>
      __layoutLength(declared[`${prefix}${side}`], {
        ...context,
        percentBase: undefined,
      }) ?? 0;
    return {
      top: resolve('top'),
      right: resolve('right'),
      bottom: resolve('bottom'),
      left: resolve('left'),
    };
  };
  const borderWidth = (side) => {
    const style = String(declared[`border-${side}-style`] ?? 'none').toLowerCase();
    if (style === 'none' || style === 'hidden') return 0;
    const width = String(declared[`border-${side}-width`] ?? 'medium').toLowerCase();
    if (width === 'medium') return 3;
    if (width === 'thin') return 1;
    if (width === 'thick') return 5;
    return __layoutLength(width, context) ?? 0;
  };

  const computed = {
    element,
    tag: String(element.localName ?? element.tagName ?? '').toLowerCase(),
    display: String(declared.display ?? 'inline')
      .trim()
      .toLowerCase(),
    position: String(declared.position ?? 'static')
      .trim()
      .toLowerCase(),
    visibility: String(declared.visibility ?? 'visible')
      .trim()
      .toLowerCase(),
    boxSizing: String(declared['box-sizing'] ?? 'content-box')
      .trim()
      .toLowerCase(),
    fontSize,
    family,
    weight,
    fontStyle,
    bold: __layoutIsBold(weight),
    italic: /italic|oblique/i.test(fontStyle),
    lineHeight,
    color: String(declared.color ?? inherited.color),
    textAlign: String(declared['text-align'] ?? 'start')
      .trim()
      .toLowerCase(),
    whiteSpace: String(declared['white-space'] ?? 'normal')
      .trim()
      .toLowerCase(),
    margin: sides('margin-'),
    padding: sides('padding-'),
    border: {
      top: borderWidth('top'),
      right: borderWidth('right'),
      bottom: borderWidth('bottom'),
      left: borderWidth('left'),
    },
    widthText: declared.width ?? 'auto',
    heightText: declared.height ?? 'auto',
    width: __layoutLength(declared.width, context),
    height: __layoutLength(declared.height, context),
    top: __layoutLength(declared.top, context),
    left: __layoutLength(declared.left, context),
    right: __layoutLength(declared.right, context),
    bottom: __layoutLength(declared.bottom, context),
    // Raw declarations stay reachable so the placement pass can resolve the
    // percentage bases that depend on the containing block.
    declared,
    context,
  };
  __layoutComputedCache.set(element, { version, computed });
  return computed;
}

function __layoutTextMetrics(text, computed) {
  const key = `${computed.family}\u0000${computed.fontSize}\u0000${computed.bold}\u0000${computed.italic}\u0000${text}`;
  const cached = __layoutTextCache.get(key);
  if (cached !== undefined) return cached;
  let metrics = null;
  try {
    metrics = JSON.parse(
      __callHost(
        'textMetrics',
        globalThis,
        JSON.stringify({
          text,
          family: computed.family,
          size: computed.fontSize,
          bold: computed.bold,
          italic: computed.italic,
        }),
      ),
    );
  } catch {
    metrics = null;
  }
  if (!metrics || !Number.isFinite(metrics.width)) {
    // Without a platform font manager the declared family's mean advance still
    // gives a page usable numbers; nothing pretends to be a browser's metrics.
    const advance = __layoutMachine.profile?.fonts?.metrics?.[computed.family] ?? 0.5;
    metrics = {
      width: Array.from(String(text)).length * advance * computed.fontSize,
      ascent: computed.fontSize * 0.8,
      descent: computed.fontSize * 0.2,
      leading: 0,
    };
  }
  if (__layoutTextCache.size > 4000) __layoutTextCache.clear();
  __layoutTextCache.set(key, metrics);
  return metrics;
}

function __layoutLineHeight(computed) {
  if (computed.lineHeight !== null) return computed.lineHeight;
  const metrics = __layoutTextMetrics('', computed);
  return Math.max(metrics.ascent + metrics.descent + metrics.leading, computed.fontSize);
}

function __layoutCollapse(text) {
  return String(text).replace(/[\t\n\r\f ]+/g, ' ');
}

const __layoutBlockLike = (computed) =>
  computed.display !== 'inline' &&
  computed.display !== 'inline-block' &&
  computed.display !== 'none';

// Used values for one containing block: percentages resolve against its
// content width (width/margins/padding) or the viewport (vertical insets).
function __layoutUsed(computed, containingWidth) {
  const { declared, context } = computed;
  const resolve = (name, base, fallback) => {
    if (declared[name] === undefined) return fallback;
    const value = __layoutLength(declared[name], { ...context, percentBase: base });
    return value === null ? fallback : value;
  };
  const side = (prefix, base, fallback) => ({
    top: resolve(`${prefix}top`, base, fallback.top),
    right: resolve(`${prefix}right`, base, fallback.right),
    bottom: resolve(`${prefix}bottom`, base, fallback.bottom),
    left: resolve(`${prefix}left`, base, fallback.left),
  });
  const width =
    computed.forcedWidth !== undefined
      ? computed.forcedWidth
      : resolve('width', containingWidth, Infinity);
  // `auto` and percentages we cannot resolve both mean "fill".
  const usedWidth = width === Infinity ? null : width;
  return {
    width: usedWidth,
    height: resolve('height', context.viewportHeight, null),
    margin: side('margin-', containingWidth, computed.margin),
    padding: side('padding-', containingWidth, computed.padding),
    top: resolve('top', context.viewportHeight, null),
    left: resolve('left', containingWidth, null),
    right: resolve('right', containingWidth, null),
    bottom: resolve('bottom', context.viewportHeight, null),
  };
}

// A block container: compute the border box, then flow its content.
function __layoutBlock(element, computed, x, y, availableWidth, boxes) {
  const used = __layoutUsed(computed, availableWidth);
  const { border } = computed;
  const margin = used.margin;
  const padding = used.padding;
  const horizontalExtra =
    margin.left + margin.right + padding.left + padding.right + border.left + border.right;
  let contentWidth;
  if (used.width !== null) {
    contentWidth =
      computed.boxSizing === 'border-box'
        ? Math.max(0, used.width - padding.left - padding.right - border.left - border.right)
        : used.width;
  } else {
    contentWidth = Math.max(0, availableWidth - horizontalExtra);
  }
  const borderX = x + margin.left;
  const borderY = y + margin.top;
  const contentX = borderX + border.left + padding.left;
  const contentY = borderY + border.top + padding.top;

  const box = {
    x: borderX,
    y: borderY,
    width: contentWidth + padding.left + padding.right + border.left + border.right,
    height: 0,
    kind: 'block',
    contentX,
    contentY,
    contentWidth,
    contentHeight: 0,
    margin,
    padding,
    border,
  };
  boxes.set(element, box);

  let cursorY = 0;
  let lastMarginBottom = 0;
  // Inline content is laid out as the runs that sit between block children, so
  // each child is visited once: a run never re-places a block already in flow.
  let run = [];
  const flushInline = () => {
    if (run.length === 0) return;
    const flow = __layoutInlineFlow(
      run,
      computed,
      contentX,
      contentY + cursorY,
      contentWidth,
      boxes,
      element,
    );
    cursorY += flow.height;
    run = [];
  };
  const nodes = element.childNodes ? Array.from(element.childNodes) : [];
  for (const node of nodes) {
    if (node.nodeType === 3) {
      if (__layoutCollapse(node.data ?? '').trim() === '') continue;
      run.push(node);
      continue;
    }
    if (node.nodeType !== 1) continue;
    const child = node;
    const childComputed = __layoutComputed(child);
    if (childComputed.display === 'none' || childComputed.visibility === 'hidden') continue;
    if (childComputed.position === 'absolute' || childComputed.position === 'fixed') {
      // Out of flow: it consumes no space and is placed against the nearest
      // positioned ancestor once that ancestor's box exists.
      __layoutAbsolute(child, childComputed, contentX, contentY, availableWidth, boxes);
      continue;
    }
    if (!__layoutBlockLike(childComputed)) {
      run.push(child);
      continue;
    }
    flushInline();
    const gap = Math.max(lastMarginBottom, childComputed.margin.top);
    const childBorderTop = cursorY + gap;
    const result = __layoutBlock(
      child,
      childComputed,
      contentX,
      contentY + childBorderTop - childComputed.margin.top,
      contentWidth,
      boxes,
    );
    cursorY = childBorderTop + result.box.height;
    lastMarginBottom = childComputed.margin.bottom;
  }
  flushInline();
  cursorY += lastMarginBottom;

  const declaredHeight = used.height;
  box.contentHeight =
    declaredHeight !== null
      ? computed.boxSizing === 'border-box'
        ? Math.max(0, declaredHeight - padding.top - padding.bottom - border.top - border.bottom)
        : declaredHeight
      : Math.max(0, cursorY);
  box.height = box.contentHeight + padding.top + padding.bottom + border.top + border.bottom;

  if (computed.position === 'relative') {
    const dx = used.left ?? (used.right !== null ? -used.right : 0);
    const dy = used.top ?? (used.bottom !== null ? -used.bottom : 0);
    if (dx !== 0 || dy !== 0) __layoutShift(element, dx, dy, boxes);
  }
  return { outerHeight: margin.top + box.height + margin.bottom, box };
}

// Normal flow inside a content box: text and inline content share line boxes,
// an inline-block that does not fit starts a new line.
function __layoutInlineFlow(runNodes, computed, startX, startY, contentWidth, boxes, owner) {
  const right = startX + contentWidth;
  let x = startX;
  let lineTop = startY;
  let lineHeight = __layoutLineHeight(computed);
  let lineHasContent = false;
  const fragments = new Map();
  const record = (node, rect) => {
    const list = fragments.get(node);
    if (list) list.push(rect);
    else fragments.set(node, [rect]);
  };
  const breakLine = () => {
    lineTop += lineHeight;
    x = startX;
    lineHasContent = false;
    lineHeight = __layoutLineHeight(computed);
  };
  const place = (node, width, height) => {
    if (lineHasContent && x + width > right) breakLine();
    lineHeight = Math.max(lineHeight, height);
    const rect = { x, y: lineTop, width, height };
    x += width;
    lineHasContent = true;
    record(node, rect);
    return rect;
  };

  const nodes = runNodes;
  for (const node of nodes) {
    if (node.nodeType === 3) {
      const text = __layoutCollapse(node.data ?? '');
      if (text.trim() === '') continue;
      const words = text.split(' ');
      for (let index = 0; index < words.length; index++) {
        const word = words[index];
        if (word === '') continue;
        if (index > 0) x += __layoutTextMetrics(' ', computed).width;
        const width = __layoutTextMetrics(word, computed).width;
        if (lineHasContent && x + width > right) {
          breakLine();
          // A leading space never starts a line.
          x = startX;
        }
        const lineBox = __layoutLineHeight(computed);
        lineHeight = Math.max(lineHeight, lineBox);
        record(node, { x, y: lineTop, width, height: lineBox });
        x += width;
        lineHasContent = true;
      }
      continue;
    }
    if (node.nodeType !== 1) continue;
    const child = node;
    const childComputed = __layoutComputed(child);
    if (childComputed.display === 'none' || childComputed.visibility === 'hidden') continue;
    if (__layoutBlockLike(childComputed)) {
      // A block inside inline content: flush the current line first.
      lineTop += lineHeight;
      x = startX;
      lineHasContent = false;
      lineHeight = __layoutLineHeight(computed);
      const result = __layoutBlock(child, childComputed, startX, lineTop, contentWidth, boxes);
      lineTop += result.outerHeight;
      continue;
    }
    if (childComputed.display === 'inline-block') {
      const intrinsic = __layoutInlineBlockWidth(child, childComputed, contentWidth);
      const outerWidth = Math.min(
        intrinsic +
          childComputed.padding.left +
          childComputed.padding.right +
          childComputed.border.left +
          childComputed.border.right +
          childComputed.margin.left +
          childComputed.margin.right,
        contentWidth,
      );
      if (lineHasContent && x + outerWidth > right) breakLine();
      const result = __layoutBlock(
        child,
        {
          ...childComputed,
          // Shrink-to-fit: the used width is the intrinsic content width plus
          // this element's own horizontal padding and border.
          forcedWidth:
            childComputed.width ??
            intrinsic + childComputed.padding.left + childComputed.padding.right,
          ...(childComputed.width === null ? { boxSizing: 'border-box' } : {}),
        },
        x,
        lineTop,
        contentWidth,
        boxes,
      );
      lineHeight = Math.max(
        lineHeight,
        result.box.height + childComputed.margin.top + childComputed.margin.bottom,
      );
      x += result.box.width + childComputed.margin.left + childComputed.margin.right;
      lineHasContent = true;
      record(child, result.box);
      continue;
    }
    // Inline element: lay its content out in this line and keep one box that
    // spans its fragments.
    const lineX = x;
    const inlineStartTop = lineTop;
    const flow = __layoutInlineFlow(
      child.childNodes ? Array.from(child.childNodes) : [],
      childComputed,
      x,
      lineTop,
      Math.max(0, right - x),
      boxes,
      child,
    );
    // An inline box with no content still has a line box of its own line height:
    // Safari reports a zero-width, line-height-tall rectangle for it.
    const empty = flow.width === 0 && flow.height === 0;
    const inlineHeight = empty ? __layoutLineHeight(childComputed) : flow.height;
    lineHeight = Math.max(lineHeight, inlineHeight);
    if (empty) record(child, { x: lineX, y: inlineStartTop, width: 0, height: inlineHeight });
    const box = {
      x: lineX,
      y: inlineStartTop,
      width: Math.max(0, flow.width),
      height: inlineHeight,
      kind: 'inline',
      contentX: lineX,
      contentY: inlineStartTop,
      contentWidth: Math.max(0, flow.width),
      contentHeight: inlineHeight,
    };
    boxes.set(child, box);
    x = lineX + Math.max(0, flow.width);
    lineHasContent = lineHasContent || flow.width > 0;
  }
  if (lineHasContent) lineTop += lineHeight;
  const height = Math.max(0, lineTop - startY);
  let maxRight = 0;
  for (const list of fragments.values())
    for (const rect of list) maxRight = Math.max(maxRight, rect.x + rect.width);
  if (owner) __layoutState.fragments.set(owner, fragments);
  return { height, width: Math.max(0, maxRight - startX) };
}

// Intrinsic width of an inline-block: its widest text run, or the width it was
// given when it has non-text content.
function __layoutInlineBlockWidth(element, computed, availableWidth) {
  if (computed.width !== null) return computed.width;
  const tag = computed.tag;
  if (
    [
      'input',
      'button',
      'textarea',
      'select',
      'img',
      'canvas',
      'iframe',
      'video',
      'audio',
      'svg',
    ].includes(tag)
  ) {
    const attribute = (name) => {
      try {
        return element.getAttribute?.(name) ?? null;
      } catch {
        return null;
      }
    };
    const measured = __layoutLength(attribute('width'), {
      fontSize: computed.fontSize,
      unitless: false,
    });
    if (measured !== null) return measured;
    if (tag === 'input' || tag === 'button') {
      const type = String(
        attribute('type') ?? (tag === 'button' ? 'submit' : 'text'),
      ).toLowerCase();
      if (type === 'submit' || type === 'button' || type === 'reset') {
        const label = tag === 'button' ? (element.textContent ?? '') : type;
        return __layoutTextMetrics(label, computed).width + 12;
      }
      if (type === 'checkbox' || type === 'radio') return 13;
      return 145;
    }
    return 0;
  }
  const text = __layoutCollapse(element.textContent ?? '').trim();
  if (!text) {
    let widest = 0;
    for (const node of Array.from(element.childNodes ?? [])) {
      if (node.nodeType !== 1) continue;
      const childComputed = __layoutComputed(node);
      if (__layoutBlockLike(childComputed)) {
        const measured = __layoutInlineBlockWidth(node, childComputed, availableWidth);
        widest = Math.max(
          widest,
          measured +
            childComputed.padding.left +
            childComputed.padding.right +
            childComputed.border.left +
            childComputed.border.right +
            childComputed.margin.left +
            childComputed.margin.right,
        );
      }
    }
    return Math.min(widest, availableWidth);
  }
  // Max-content is the whole run on one line, shaped as one string: measuring
  // word by word would lose the kerning between them.
  const maxContent = __layoutTextMetrics(text, computed).width;
  return Math.min(availableWidth, maxContent);
}

function __layoutAbsolute(element, computed, originX, originY, availableWidth, boxes) {
  let containing = null;
  for (let node = element.parentElement; node; node = node.parentElement) {
    if (__layoutComputed(node).position !== 'static') {
      containing = node;
      break;
    }
  }
  let x = computed.position === 'fixed' ? 0 : originX;
  let y = computed.position === 'fixed' ? 0 : originY;
  let width = availableWidth;
  if (computed.position === 'absolute' && containing) {
    const containingBox = boxes.get(containing);
    if (containingBox) {
      x = containingBox.contentX;
      y = containingBox.contentY;
      width = containingBox.contentWidth;
    }
  }
  const used = __layoutUsed(computed, width);
  const result = __layoutBlock(
    element,
    computed,
    x + (used.left ?? 0),
    y + (used.top ?? 0),
    width,
    boxes,
  );
  return result;
}

function __layoutShift(element, dx, dy, boxes) {
  const seen = new Set();
  const shift = (node) => {
    if (!node || seen.has(node)) return;
    seen.add(node);
    const box = boxes.get(node);
    if (box) {
      box.x += dx;
      box.y += dy;
      box.contentX += dx;
      box.contentY += dy;
    }
    for (const child of Array.from(node.childNodes ?? [])) if (child.nodeType === 1) shift(child);
  };
  shift(element);
}

function __layoutRun() {
  const document = globalThis.document;
  if (!document) return new Map();
  if (!__layoutState.observer && typeof MutationObserver === 'function') {
    __layoutState.observer = new MutationObserver(__layoutInvalidate);
    try {
      __layoutState.observer.observe(document, {
        subtree: true,
        childList: true,
        attributes: true,
        characterData: true,
      });
    } catch {
      __layoutState.observer = null;
    }
  }
  if (__layoutState.boxes) return __layoutState.boxes;
  const boxes = new Map();
  __layoutState.fragments = new Map();
  __layoutState.boxes = boxes;
  const root = document.documentElement ?? document.body;
  if (!root) return boxes;
  __layoutBlock(root, __layoutComputed(root), 0, 0, __layoutViewport().width, boxes);
  return boxes;
}

function __layoutBox(element) {
  if (!element || element.nodeType !== 1) return null;
  return __layoutRun().get(element) ?? null;
}

function __layoutFragments(element) {
  __layoutRun();
  const fragments = __layoutState.fragments?.get(element);
  if (!fragments || fragments.size === 0) return null;
  const rects = [];
  for (const list of fragments.values()) rects.push(...list);
  return rects.length ? rects : null;
}

// What `getComputedStyle` reports: used values where layout has them, the
// declared value otherwise.
function __layoutComputedProperty(element, name) {
  const computed = __layoutComputed(element);
  const box = __layoutBox(element);
  // WebKit lays out in 1/64 px units and reports the quantised length, while a
  // specified length such as `font-size` keeps the value it was given.
  const px = (value) => `${Math.round(value * 64) / 64}px`;
  const specified = (value) => `${Math.round(value * 10000) / 10000}px`;
  switch (name) {
    case 'display':
      return computed.display;
    case 'position':
      return computed.position;
    case 'visibility':
      return computed.visibility;
    case 'box-sizing':
      return computed.boxSizing;
    case 'width':
      return box && box.kind === 'block' ? px(box.contentWidth) : computed.widthText;
    case 'height':
      return box && box.kind === 'block' ? px(box.contentHeight) : computed.heightText;
    case 'font-size':
      return specified(computed.fontSize);
    case 'font-family':
      return computed.family;
    case 'font-weight':
      return computed.weight;
    case 'font-style':
      return computed.fontStyle;
    case 'line-height':
      return computed.lineHeight === null ? 'normal' : px(computed.lineHeight);
    case 'color':
      return computed.color;
    case 'text-align':
      return computed.textAlign;
    case 'white-space':
      return computed.whiteSpace;
    case 'margin-top':
    case 'margin-right':
    case 'margin-bottom':
    case 'margin-left':
      return px((box?.margin ?? computed.margin)[name.slice('margin-'.length)]);
    case 'padding-top':
    case 'padding-right':
    case 'padding-bottom':
    case 'padding-left':
      return px((box?.padding ?? computed.padding)[name.slice('padding-'.length)]);
    case 'border-top-width':
    case 'border-right-width':
    case 'border-bottom-width':
    case 'border-left-width':
      return px(computed.border[name.slice('border-'.length).replace('-width', '')]);
    case 'top':
      return computed.top === null ? 'auto' : px(computed.top);
    case 'left':
      return computed.left === null ? 'auto' : px(computed.left);
    case 'right':
      return computed.right === null ? 'auto' : px(computed.right);
    case 'bottom':
      return computed.bottom === null ? 'auto' : px(computed.bottom);
    default:
      return undefined;
  }
}

Object.defineProperty(globalThis, '__brimpInstallLayout', {
  value: undefined,
  writable: false,
  enumerable: false,
  configurable: true,
});
Object.defineProperty(globalThis, '__brimpTakeLayout', {
  value: () => ({
    box: __layoutBox,
    fragments: __layoutFragments,
    computed: __layoutComputed,
    invalidate: __layoutInvalidate,
    defaultFamily: () => __layoutMachine.defaultFamily,
  }),
  writable: false,
  enumerable: false,
  configurable: true,
});
// css_style.js owns the computed-style object; it takes the cascade from here.
globalThis.__brimpInstallComputedStyleSource?.(__layoutComputedProperty);
