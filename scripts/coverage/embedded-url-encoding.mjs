import { createServer } from 'node:http';
import { embeddedURLFields } from './embedded-url-cases.mjs';
export async function measureEmbeddedURLEncoding(page) {
  const labels = ['UTF-8', 'windows-1252', 'Big5', 'ISO-2022-JP'];
  const server = createServer((req, res) => {
    const encoding = labels[Number(req.url.slice(1))] ?? 'UTF-8';
    res.writeHead(200, { 'Content-Type': `text/html; charset=${encoding}` });
    res.end(`<!doctype html><meta charset="${encoding}"><body>`);
  });
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  try {
    const results = [];
    for (let i = 0; i < labels.length; i++) {
      await page.get(`http://127.0.0.1:${server.address().port}/${i}`);
      results.push(
        await page.evaluate(`(()=>{
        const results=[],value='https://example.test/é?X　☃中é🦀#é';
        for(const[tag,name]of ${JSON.stringify(embeddedURLFields)})for(const kind of ['attribute','property']){
          const e=document.createElement(tag),attribute=name.toLowerCase();
          if(kind==='attribute')e.setAttribute(attribute,value);else e[name]=value;
          const original=e[name],clone=e.cloneNode()[name],raw=e.getAttribute(attribute);
          Object.defineProperty(e,'ownerDocument',{get(){throw Error('public owner')}});
          Object.defineProperty(document,'characterSet',{configurable:true,get(){throw Error('public encoding')}});
          let canonical;try{canonical=e[name]}finally{delete document.characterSet}
          document.implementation.createHTMLDocument().adoptNode(e);
          results.push([tag,name,kind,original,clone,raw,canonical,e[name]]);
        }
        return results;
      })()`),
      );
    }
    return results;
  } finally {
    await new Promise((resolve) => server.close(resolve));
  }
}
