import test from 'node:test';
import assert from 'node:assert/strict';
import { createSession } from '../../../scripts/coverage/session.mjs';

async function evaluate(source) {
  const session = await createSession();
  try {
    return await (await session.newPage()).evaluate(source);
  } finally {
    await session.close();
  }
}

test('link and script properties reflect canonical attributes through cloning', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const failures=[];
    for(const [tag,strings] of [['link',['rel','media','integrity','hreflang','type','charset','rev','target','imageSizes','imageSrcset']],
      ['script',['type','charset','integrity','event','htmlFor']]]) {
      const e=document.createElement(tag),ctor=tag==='link'?HTMLLinkElement:HTMLScriptElement;
      if(!(e instanceof ctor)||Object.prototype.toString.call(e)!=='[object '+ctor.name+']')failures.push(tag+' identity');
      for(const name of strings) {
        const attribute=name==='htmlFor'?'for':name.toLowerCase();
        if(e[name]!=='')failures.push(name+' default');
        e[name]='sample value';
        if(e.getAttribute(attribute)!=='sample value'||e.cloneNode()[name]!=='sample value')failures.push(name+' reflection');
        e.setAttribute(attribute,'changed');if(e[name]!=='changed')failures.push(name+' mutation');
        e.removeAttribute(attribute);if(e[name]!=='')failures.push(name+' removal');
      }
      e.crossOrigin='USE-CREDENTIALS';if(e.crossOrigin!=='use-credentials')failures.push(tag+' cors');
      e.crossOrigin='invalid';if(e.crossOrigin!=='anonymous')failures.push(tag+' invalid cors');
      e.crossOrigin=null;if(e.crossOrigin!==null||e.hasAttribute('crossorigin'))failures.push(tag+' null cors');
      e.referrerPolicy='NO-REFERRER';if(e.referrerPolicy!=='no-referrer'||e.getAttribute('referrerpolicy')!=='NO-REFERRER')failures.push(tag+' policy');
      e.referrerPolicy='invalid';if(e.referrerPolicy!=='')failures.push(tag+' invalid policy');
    }
    const link=document.createElement('link');link.as='SCRIPT';if(link.as!=='script')failures.push('as');
    link.as='invalid';if(link.as!=='')failures.push('invalid as');
    for(const name of ['defer','noModule']) {
      const script=document.createElement('script');if(script[name]!==false)failures.push(name+' default');
      script[name]=true;if(!script.hasAttribute(name.toLowerCase())||!script.cloneNode()[name])failures.push(name+' set');
      script[name]=false;if(script.hasAttribute(name.toLowerCase()))failures.push(name+' clear');
    }
    return failures;
  })()`),
    [],
  );
});

test('link tokens and resource URLs retain identity and resolve against the owning document', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const doc=document.implementation.createHTMLDocument();doc.head.innerHTML='<base href="https://assets.test/dir/">';
    const link=doc.createElement('link'),script=doc.createElement('script');
    link.href='theme.css';script.src='app.js';
    const urls=[link.href,script.src];doc.head.firstChild.href='https://changed.test/';
    urls.push(link.href,script.src);
    const rel=link.relList,sizes=link.sizes;link.rel='preload preload stylesheet';sizes.add('16x16','32x32');
    rel.remove('stylesheet');
    return [urls,rel===link.relList,sizes===link.sizes,[...rel],link.rel,[...sizes],link.getAttribute('sizes'),
      link.cloneNode().sizes.value];
  })()`),
    [
      [
        'https://assets.test/dir/theme.css',
        'https://assets.test/dir/app.js',
        'https://changed.test/theme.css',
        'https://changed.test/app.js',
      ],
      true,
      true,
      ['preload'],
      'preload',
      ['16x16', '32x32'],
      '16x16 32x32',
      '16x16 32x32',
    ],
  );
});

test('script text uses direct text children and executes that source when inserted', async () => {
  const session = await createSession();
  try {
    const page = await session.newPage();
    assert.deepEqual(
      await page.evaluate(`(() => {
      const script=document.createElement('script'),nested=document.createElement('span');
      nested.textContent='throw new Error("nested text must not run");';
      script.append(document.createTextNode('window.scriptTextResult=42;'),nested,document.createComment('ignored'));
      const first=script.text,clone=script.cloneNode(true).text;
      document.body.append(script);
      const replacement=document.createElement('script');replacement.append(document.createElement('span'));
      replacement.text='window.scriptReplacement=7;';
      const state=[replacement.text,replacement.textContent,replacement.childNodes.length,replacement.firstChild.nodeType];
      document.body.append(replacement);
      return [first,clone,state];
    })()`),
      [
        'window.scriptTextResult=42;',
        'window.scriptTextResult=42;',
        ['window.scriptReplacement=7;', 'window.scriptReplacement=7;', 1, 3],
      ],
    );
    assert.deepEqual(
      await page.evaluate('[window.scriptTextResult,window.scriptReplacement]'),
      [42, 7],
    );
  } finally {
    await session.close();
  }
});
