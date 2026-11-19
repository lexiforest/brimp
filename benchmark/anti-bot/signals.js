// The signals the pyv8c challenge write-ups say a sensor reads, measured the same
// way in Brimp and in the browser the profile claims.
//
// `signalsStart` begins the time-based observations, `signalsRead` reports them;
// the two calls are separated by a real pause so a frame counter can be turned
// into a rate without the probe itself blocking the page.
globalThis.__signals = { frames: [], deltas: [] };

globalThis.signalsStart = () => {
  const state = globalThis.__signals;
  const tick = (timestamp) => {
    state.frames.push(timestamp);
    if (state.frames.length < 10000) requestAnimationFrame(tick);
  };
  requestAnimationFrame(tick);
  const deadline = performance.now() + 2500;
  const sample = () => {
    state.deltas.push(performance.now());
    if (performance.now() < deadline) setTimeout(sample, 0);
  };
  sample();
  return true;
};

globalThis.signalsRead = () => {
  const state = globalThis.__signals;
  const frames = state.frames;
  const elapsed = frames.length > 1 ? frames[frames.length - 1] - frames[0] : 0;
  const fps = elapsed > 0 ? ((frames.length - 1) * 1000) / elapsed : 0;

  // `performance.now()` is quantised to 100 µs in a browser; a raw clock leaks a
  // finer resolution that is itself a fingerprint.
  const subMillisecond = state.deltas
    .map((value) => Math.round((value - Math.floor(value)) * 10000) / 10000)
    .filter((fraction) => fraction !== 0);
  const quantised =
    subMillisecond.length > 0 &&
    subMillisecond.every((fraction) => Math.abs(fraction * 10 - Math.round(fraction * 10)) < 1e-9);

  // What a prefix/engine check counts: the enumerable members the style object
  // exposes, and the accessor count on its prototype.
  const style = document.createElement('div').style;
  let styleMembers = 0;
  for (const key in style) styleMembers += 1;
  const stylePrototype = Object.getOwnPropertyNames(Object.getPrototypeOf(style)).length;

  const copy = document.getElementById('copy');
  const logo = document.getElementById('logo');
  const empty = document.getElementById('empty');
  const rect = copy.getBoundingClientRect();
  const paragraphStyle = getComputedStyle(copy);
  const headingStyle = getComputedStyle(document.querySelector('h1'));
  const listStyle = getComputedStyle(document.querySelector('ul'));

  return JSON.stringify({
    rafFps: Math.round(fps),
    rafFrames: frames.length,
    nowQuantised: quantised,
    nowDistinctFractions: new Set(subMillisecond).size,
    styleMembers,
    stylePrototype,
    voices: typeof speechSynthesis === 'undefined' ? null : speechSynthesis.getVoices().length,
    logoSize: [logo.offsetWidth, logo.offsetHeight],
    emptyInline: (() => {
      const box = empty.getBoundingClientRect();
      return [Math.round(box.width), Math.round(box.height), getComputedStyle(empty).display];
    })(),
    rectKind: [
      Object.prototype.toString.call(rect),
      rect instanceof DOMRect,
      typeof rect.toJSON,
      Object.getPrototypeOf(rect) === DOMRect.prototype,
    ],
    blocks: [paragraphStyle.display, headingStyle.display, listStyle.display],
    uaMargins: [paragraphStyle.marginTop, headingStyle.marginTop, listStyle.paddingLeft],
    linkedSheet: [
      document.styleSheets.length,
      paragraphStyle.color,
      (document.styleSheets[0] || {}).href || null,
    ],
    hasWebGL: (() => {
      const gl = document.createElement('canvas').getContext('webgl');
      return gl ? [gl.getParameter(37445), gl.getParameter(37446)] : null;
    })(),
  });
};
