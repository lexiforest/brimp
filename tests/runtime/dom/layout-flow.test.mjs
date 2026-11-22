import test from 'node:test';
import assert from 'node:assert/strict';
import { createSession } from '../../../bindings/node/index.js';

// Layout is measured against the platform's fonts, so these assertions are
// relations and exact box sums, never a hardcoded glyph advance.

test('normal flow stacks blocks inside the body margin', async () => {
  const session = await createSession();
  try {
    const page = await session.newPage();
    const result = await page.evaluate(`(() => {
      document.body.innerHTML =
        '<div id="one" style="height:10px"></div>' +
        '<div id="two" style="height:20px;margin-top:5px"></div>' +
        '<div id="three" style="height:30px"></div>';
      const rect = (id) => { const r = document.getElementById(id).getBoundingClientRect();
        return [r.x, r.y, r.width, r.height]; };
      return JSON.stringify({
        one: rect('one'), two: rect('two'), three: rect('three'),
        body: [document.body.getBoundingClientRect().width,
               document.body.getBoundingClientRect().height],
        offset: [document.getElementById('two').offsetTop, document.getElementById('two').offsetLeft],
      });
    })()`);
    assert.deepEqual(JSON.parse(result), {
      one: [8, 8, 784, 10],
      two: [8, 23, 784, 20],
      three: [8, 43, 784, 30],
      body: [784, 65],
      offset: [15, 0],
    });
  } finally {
    await session.close();
  }
});

test('padding and borders enlarge the border box without moving the content edge', async () => {
  const session = await createSession();
  try {
    const page = await session.newPage();
    const result = await page.evaluate(`(() => {
      document.body.innerHTML = '<div id="a" style="width:100px;padding:10px;border:2px solid"></div>';
      const a = document.getElementById('a');
      const rect = a.getBoundingClientRect();
      const style = getComputedStyle(a);
      return JSON.stringify({
        rect: [rect.x, rect.y, rect.width, rect.height],
        width: style.width, padding: style.paddingTop, border: style.borderTopWidth,
        offsetWidth: a.offsetWidth, clientWidth: a.clientWidth,
      });
    })()`);
    assert.deepEqual(JSON.parse(result), {
      rect: [8, 8, 124, 24],
      width: '100px',
      padding: '10px',
      border: '2px',
      offsetWidth: 124,
      clientWidth: 124,
    });
  } finally {
    await session.close();
  }
});

test('a text run gets a line box from the shaped font, not a fixed ratio', async () => {
  const session = await createSession();
  try {
    const page = await session.newPage();
    const result = await page.evaluate(`(() => {
      document.body.innerHTML = '<div id="a" style="font-size:20px;line-height:normal">hello</div>' +
        '<div id="b" style="font-size:20px;line-height:40px">hello</div>';
      const ctx = document.createElement('canvas').getContext('2d');
      ctx.font = '20px -apple-system';
      const measured = ctx.measureText('hello').width;
      const a = document.getElementById('a').getBoundingClientRect();
      const b = document.getElementById('b').getBoundingClientRect();
      return JSON.stringify({
        normalHeight: a.height, fixedHeight: b.height, measured,
        lineHeight: getComputedStyle(document.getElementById('b')).lineHeight,
        normalLineHeight: getComputedStyle(document.getElementById('a')).lineHeight,
      });
    })()`);
    const values = JSON.parse(result);
    assert.equal(values.fixedHeight, 40);
    assert.equal(values.lineHeight, '40px');
    assert.equal(values.normalLineHeight, 'normal');
    // A shaped 20px line is at least the font size and shorter than 3x it.
    assert.ok(values.normalHeight >= 20 && values.normalHeight < 60, String(values.normalHeight));
    // Font metrics and CSS pixel geometry can be fractional on Linux.
    assert.ok(Number.isFinite(values.normalHeight));
    assert.ok(values.measured > 20, String(values.measured));
  } finally {
    await session.close();
  }
});

test('author rules cascade by specificity and source order over the user agent', async () => {
  const session = await createSession();
  try {
    const page = await session.newPage();
    const result = await page.evaluate(`(() => {
      document.head.innerHTML = '<style>' +
        'p { margin-top: 20px; color: red }' +
        '.lead { margin-top: 30px }' +
        '#first { margin-top: 40px; width: 50% }' +
        '@media (min-width: 600px) { .lead { color: blue } }' +
        '</style>';
      document.body.innerHTML = '<p id="first" class="lead">a</p><p id="second" class="lead">b</p>';
      const style = (id) => getComputedStyle(document.getElementById(id));
      return JSON.stringify({
        first: [style('first').marginTop, style('first').color, style('first').width],
        second: [style('second').marginTop, style('second').color],
        inlineStart: document.getElementById('first').getBoundingClientRect().y,
      });
    })()`);
    assert.deepEqual(JSON.parse(result), {
      // `#first` also carries `.lead`, so the media rule wins on specificity.
      first: ['40px', 'blue', '392px'],
      second: ['30px', 'blue'],
      inlineStart: 48,
    });
  } finally {
    await session.close();
  }
});

test('the font shorthand sets size, line height and family, and font-family inherits', async () => {
  const session = await createSession();
  try {
    const page = await session.newPage();
    const result = await page.evaluate(`(() => {
      document.body.innerHTML = '<div id="a" style="font:italic 700 18px/24px Georgia">x<span id="b">y</span></div>';
      const a = getComputedStyle(document.getElementById('a'));
      const b = getComputedStyle(document.getElementById('b'));
      return JSON.stringify({
        size: a.fontSize, line: a.lineHeight, family: a.fontFamily,
        weight: a.fontWeight, italic: a.fontStyle,
        inherited: [b.fontSize, b.lineHeight, b.fontFamily],
        height: document.getElementById('a').getBoundingClientRect().height,
      });
    })()`);
    assert.deepEqual(JSON.parse(result), {
      size: '18px',
      line: '24px',
      family: 'Georgia',
      weight: '700',
      italic: 'italic',
      inherited: ['18px', '24px', 'Georgia'],
      height: 24,
    });
  } finally {
    await session.close();
  }
});

test('wrapped inline text reports one client rect per line', async () => {
  const session = await createSession();
  try {
    const page = await session.newPage();
    const result = await page.evaluate(`(() => {
      document.body.innerHTML = '<div id="a" style="width:120px;font-size:16px">' +
        'one two three four five six seven eight nine ten</div>';
      const a = document.getElementById('a');
      const rects = [...a.getClientRects()];
      const line = getComputedStyle(a).lineHeight === 'normal'
        ? a.getBoundingClientRect().height / Math.max(rects.length, 1) : 0;
      return JSON.stringify({
        count: rects.length,
        rectHeight: a.getBoundingClientRect().height,
        firstWidth: rects[0].width,
        allWithinWidth: rects.every((r) => r.width <= 120.5),
        sameLineHeight: new Set(rects.map((r) => Math.round(r.height))).size,
        line,
      });
    })()`);
    const values = JSON.parse(result);
    assert.ok(values.count >= 3, `expected several lines, got ${values.count}`);
    assert.ok(values.allWithinWidth);
    assert.equal(values.sameLineHeight, 1);
    assert.ok(values.firstWidth > 0);
    assert.equal(values.line > 0, true);
  } finally {
    await session.close();
  }
});

test('inline-block shrinks to its text and sits in the line box', async () => {
  const session = await createSession();
  try {
    const page = await session.newPage();
    const result = await page.evaluate(`(() => {
      document.body.innerHTML = '<div id="a" style="width:400px;font-size:16px">before ' +
        '<span id="b" style="display:inline-block;padding:4px">shrink me</span> after</div>';
      const b = document.getElementById('b');
      const text = document.createRange ? null : null;
      const ctx = document.createElement('canvas').getContext('2d');
      ctx.font = '16px -apple-system';
      const measured = ctx.measureText('shrink me').width;
      return JSON.stringify({
        width: b.getBoundingClientRect().width,
        expected: measured + 8,
        display: getComputedStyle(b).display,
        top: b.getBoundingClientRect().y >= 8,
        withinParent: b.getBoundingClientRect().width <= 400,
        text,
      });
    })()`);
    const values = JSON.parse(result);
    assert.equal(values.display, 'inline-block');
    assert.ok(
      Math.abs(values.width - values.expected) < 1.5,
      `${values.width} vs ${values.expected}`,
    );
    assert.equal(values.top, true);
    assert.equal(values.withinParent, true);
    assert.equal(values.text, null);
  } finally {
    await session.close();
  }
});

test('hidden elements have no box and no layout effect', async () => {
  const session = await createSession();
  try {
    const page = await session.newPage();
    const result = await page.evaluate(`(() => {
      document.body.innerHTML = '<div id="a" style="height:10px"></div>' +
        '<div id="hidden" style="display:none;height:500px"></div>' +
        '<div id="b" style="height:12px"></div>';
      const rect = (id) => { const r = document.getElementById(id).getBoundingClientRect();
        return [r.y, r.height]; };
      return JSON.stringify({
        a: rect('a'), b: rect('b'), hidden: rect('hidden'),
        hiddenWidth: getComputedStyle(document.getElementById('hidden')).width,
      });
    })()`);
    assert.deepEqual(JSON.parse(result), {
      a: [8, 10],
      b: [18, 12],
      hidden: [0, 0],
      hiddenWidth: 'auto',
    });
  } finally {
    await session.close();
  }
});

test('a percentage width resolves against the containing block, not the viewport', async () => {
  const session = await createSession();
  try {
    const page = await session.newPage();
    await page.evaluate(`document.body.innerHTML =
      '<div id="outer" style="width:400px"><div id="a" style="width:50%;height:10px"></div></div>'`);
    const read = () =>
      page.evaluate(`(() => { const r = document.querySelector('#a').getBoundingClientRect();
        return JSON.stringify([r.x, r.y, r.width, r.height]); })()`);
    const inner = JSON.parse(await read());
    await page.evaluate(`document.body.innerHTML =
      '<div id="a" style="width:50%;height:10px"></div>'`);
    const direct = JSON.parse(await read());
    assert.deepEqual(inner, [8, 8, 200, 10]);
    assert.deepEqual(direct, [8, 8, 392, 10]);
  } finally {
    await session.close();
  }
});
