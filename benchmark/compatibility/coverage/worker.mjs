import { createRequire } from 'node:module';
import { readFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { probes } from './probes.mjs';
const require = createRequire(import.meta.url),
  engine = process.argv[2];
const html = '<!doctype html><html><head><title>coverage</title></head><body></body></html>';
const url = 'https://coverage.test/';
const result = { engine, status: 'running', probes: [], inventory: null };
let create, close;
const inventory = `(() => {
 const names=['Node','Element','Document','DocumentFragment','CharacterData','Text','Attr','NamedNodeMap','HTMLCollection','Range','Selection','TreeWalker','NodeIterator','HTMLInputElement','HTMLFormElement','HTMLSelectElement','HTMLTemplateElement','CustomElementRegistry','ShadowRoot','CSSStyleSheet','SVGSVGElement'];
 const output={};
 for(const name of names){const ctor=globalThis[name];if(typeof ctor!=='function'){output[name]=null;continue;}const members=new Set();for(let p=ctor.prototype;p;p=Object.getPrototypeOf(p)){for(const key of Object.getOwnPropertyNames(p))members.add(key);}output[name]=[...members].sort();}
 return output;
})()`;
try {
  if (engine === 'chromium') {
    const { chromium } = await import('playwright');
    const browser = await chromium.launch({ headless: true });
    result.version = browser.version();
    close = () => browser.close();
    create = async () => {
      const context = await browser.newContext();
      await context.route('**/*', (r) => r.fulfill({ contentType: 'text/html', body: html }));
      const page = await context.newPage();
      await page.goto(url);
      return { evaluate: (s) => page.evaluate(s), close: () => context.close() };
    };
  } else if (engine === 'brimp') {
    const api = await import('../../../bindings/node/index.js');
    const session = await api.createSession({ timeout: 3000 });
    close = () => session.close();
    result.nativeSha256 = createHash('sha256')
      .update(await readFile(new URL('../../../bindings/node/brimp.node', import.meta.url)))
      .digest('hex');
    result.plugins = [];
    create = async () => {
      const page = await session.newPage();
      await page.route('*', (r) =>
        r.fulfill({ body: html, headers: { 'content-type': 'text/html' } }),
      );
      await page.get(url);
      return { evaluate: (s) => page.evaluate(s), close: () => page.close() };
    };
  } else if (engine === 'jsdom') {
    const { JSDOM } = await import('jsdom');
    result.version = require('jsdom/package.json').version;
    create = async () => {
      const dom = new JSDOM(html, { url, runScripts: 'outside-only' });
      return { evaluate: (s) => dom.window.eval(s), close: () => dom.window.close() };
    };
  } else if (engine === 'happy-dom') {
    const { Window } = await import('happy-dom');
    result.version = require('happy-dom/package.json').version;
    create = async () => {
      const window = new Window({ url, settings: { enableJavaScriptEvaluation: true } });
      window.document.write(html);
      return { evaluate: (s) => window.eval(s), close: () => window.happyDOM.close() };
    };
  } else throw Error('Unknown engine');
  const first = await create();
  try {
    result.inventory = await first.evaluate(inventory);
  } finally {
    await first.close();
  }
  for (const probe of probes) {
    const row = { id: probe.id };
    let realm;
    try {
      realm = await create();
      const source = `(async()=>{try{return {ok:true,value:await ${probe.source}}}catch(e){return {ok:false,error:{name:e.name,message:String(e.message)}}}})()`;
      row.observation = JSON.parse(JSON.stringify(await realm.evaluate(source)));
      row.status = 'completed';
    } catch (e) {
      row.status = 'execution_error';
      row.failure = { name: e.name, message: e.message };
    } finally {
      await realm?.close();
    }
    result.probes.push(row);
    process.send?.({ type: 'progress', result });
  }
  result.status = 'completed';
} catch (e) {
  result.status = 'setup_error';
  result.failure = { name: e.name, message: e.message };
} finally {
  try {
    await close?.();
  } catch (e) {
    result.status = 'cleanup_error';
    result.failure = { message: e.message };
  }
  process.send?.({ type: 'result', result });
  process.disconnect?.();
}
