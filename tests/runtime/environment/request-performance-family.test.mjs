import test from 'node:test';
import assert from 'node:assert/strict';
import { createSession } from '../../../bindings/node/index.js';

async function evaluate(source) {
  const session = await createSession({ clock: { mode: 'logical' } });
  try {
    return await (await session.newPage()).evaluate(source);
  } finally {
    await session.close();
  }
}

test('request bodies, clones, metadata and headers support a local upload workflow', async () => {
  assert.deepEqual(
    await evaluate(`(async () => {
    const h = new Headers([['Z-Last',' z '],['Content-Type','application/json']]);
    h.append('Z-LAST','tail'); h.set('X-Remove','yes'); h.delete('x-remove');
    const visited = []; h.forEach((value,name,self)=>visited.push([name,value,self===h]));
    const abort = new AbortController();
    const request = new Request('https://example.test/upload',{method:'post',headers:h,body:'{"answer":42}',signal:abort.signal});
    const clone = request.clone(), binary = request.clone(), blob = request.clone();
    const json = await request.json();
    let consumed = false; try { await request.text(); } catch(e) { consumed = e instanceof TypeError; }
    abort.abort('stop');
    return [[...h.keys()],[...h.values()],[...h.entries()],visited,h.has('x-remove'),
      request.url,request.method,request.headers.get('z-last'),json.answer,request.bodyUsed,consumed,
      await clone.text(),new TextDecoder().decode(await binary.arrayBuffer()),await (await blob.blob()).text(),
      clone.signal.aborted,clone.signal.reason,request.destination,request.referrer,request.referrerPolicy,
      request.mode,request.credentials,request.cache,request.redirect,request.integrity,request.keepalive,
      'isHistoryNavigation' in request];
  })()`),
    [
      ['content-type', 'z-last'],
      ['application/json', 'z, tail'],
      [
        ['content-type', 'application/json'],
        ['z-last', 'z, tail'],
      ],
      [
        ['content-type', 'application/json', true],
        ['z-last', 'z, tail', true],
      ],
      false,
      'https://example.test/upload',
      'POST',
      'z, tail',
      42,
      true,
      true,
      '{"answer":42}',
      '{"answer":42}',
      '{"answer":42}',
      true,
      'stop',
      '',
      'about:client',
      '',
      'cors',
      'same-origin',
      'default',
      'follow',
      '',
      false,
      false,
    ],
  );
});

test('request formData parses URL-encoded bodies and labels unsupported formats as placeholders', async () => {
  assert.deepEqual(
    await evaluate(`(async () => {
    const request = new Request('https://example.test/form',{method:'POST',body:new URLSearchParams('name=Ada+Lovelace&tag=a&tag=b')});
    const form = await request.formData();
    let consumed = false; try { await request.formData(); } catch(e) { consumed = e instanceof TypeError; }
    const multipart = new Request('https://example.test/form',{method:'POST',body:new FormData()});
    return [form instanceof FormData,form.get('name'),form.getAll('tag'),request.bodyUsed,consumed,
      multipart.formData()===undefined,multipart.bodyUsed];
  })()`),
    [true, 'Ada Lovelace', ['a', 'b'], true, true, true, false],
  );
});

test('performance marks and measures coexist with explicit resource/navigation placeholders', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const first=performance.mark('begin',{startTime:2}),last=performance.mark('end',{startTime:7});
    const span=performance.measure('work','begin','end');
    let called=0;performance.onresourcetimingbufferfull=()=>called++;
    performance.dispatchEvent(new Event('resourcetimingbufferfull'));
    performance.onresourcetimingbufferfull=null;
    performance.dispatchEvent(new Event('resourcetimingbufferfull'));
    const noops=performance.clearResourceTimings()===undefined&&performance.setResourceTimingBufferSize(1)===undefined;
    const result=[first instanceof PerformanceEntry,last.entryType,span.toJSON(),
      performance.getEntries().length,performance.getEntriesByName('work')[0]===span,performance.getEntriesByType('mark').length,
      noops,performance.eventCounts,performance.navigation===performance.navigation,performance.navigation.toJSON(),
      PerformanceNavigation.TYPE_NAVIGATE,PerformanceNavigation.TYPE_RELOAD,PerformanceNavigation.TYPE_BACK_FORWARD,
      PerformanceNavigation.TYPE_RESERVED,performance.navigation.TYPE_RELOAD,called,
      PerformancePaintTiming.prototype instanceof PerformanceEntry,performance.now(),performance.toJSON().timeOrigin===performance.timeOrigin,
      'memory' in performance];
    performance.clearMarks();performance.clearMeasures('work');result.push(performance.getEntries().length);
    return result;
  })()`),
    [
      true,
      'mark',
      { name: 'work', entryType: 'measure', startTime: 2, duration: 5, detail: null },
      3,
      true,
      2,
      true,
      '',
      true,
      { type: 0, redirectCount: 0 },
      0,
      1,
      2,
      255,
      1,
      1,
      true,
      0,
      true,
      false,
      0,
    ],
  );
});
