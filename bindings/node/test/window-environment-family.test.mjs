import test, {before, after} from 'node:test';
import assert from 'node:assert/strict';
import {createServer} from 'node:http';
import {once} from 'node:events';
import {createSession} from '../index.js';

let server, origin, reloads=0;
before(async () => {
  server=createServer((request,response)=>{
    response.setHeader('content-type','text/html');
    const url=new URL(request.url,'http://fixture.test');let script='';
    if(url.pathname==='/start')script=`location.pathname='/query'`;
    else if(url.pathname==='/query')script=url.search?`document.location='/window'`:`location.search='value=next'`;
    else if(url.pathname==='/window')script=`window.location='/done#finish'`;
    else if(url.pathname==='/reload'&&++reloads===1)script='location.reload()';
    response.end(`<!doctype html><title>environment</title><body><p>fixture</p>${script?`<script>${script}</script>`:''}</body>`);
  }).listen(0,'127.0.0.1');await once(server,'listening');origin=`http://127.0.0.1:${server.address().port}`;
});
after(async()=>{server.closeAllConnections();await new Promise(resolve=>server.close(resolve));});

async function pageWith(callback,path='/history'){
  const session=await createSession();try{return await callback(await session.get(origin+path));}finally{await session.close();}
}

test('Screen, viewport and bars expose one fixed profile without rendering or automatic events',async()=>{
  assert.deepEqual(await pageWith(page=>page.evaluate(`(() => {
    let events=0;screen.orientation.onchange=()=>events++;visualViewport.onresize=()=>events++;visualViewport.onscroll=()=>events++;
    resizeTo(1200,900);scrollTo(20,30);const automatic=events;
    screen.orientation.dispatchEvent(new Event('change'));visualViewport.dispatchEvent(new Event('resize'));visualViewport.dispatchEvent(new Event('scroll'));
    visualViewport.onscroll=null;visualViewport.dispatchEvent(new Event('scroll'));
    const bars=[locationbar,menubar,personalbar,scrollbars,statusbar,toolbar];
    let illegal=0;for(const C of [Screen,ScreenOrientation,VisualViewport,BarProp])try{new C()}catch(e){if(e.name==='TypeError')illegal++}
    return [[screen.width,screen.height,screen.availWidth,screen.availHeight,screen.availLeft,screen.availTop,screen.colorDepth,screen.pixelDepth],
      [innerWidth,innerHeight,outerWidth,outerHeight,devicePixelRatio,screenX,screenY,screenLeft,screenTop,scrollX,scrollY,pageXOffset,pageYOffset],
      [visualViewport.width,visualViewport.height,visualViewport.offsetLeft,visualViewport.offsetTop,visualViewport.pageLeft,visualViewport.pageTop,visualViewport.scale],
      [screen.orientation.angle,screen.orientation.type],bars.every(b=>b instanceof BarProp&&b.visible),new Set(bars).size,
      screen instanceof Screen,visualViewport instanceof VisualViewport,screen.orientation instanceof ScreenOrientation,
      screen===window.screen,visualViewport===window.visualViewport,locationbar===window.locationbar,
      automatic,events,illegal,typeof screen.orientation.lock,typeof screen.orientation.unlock,
      'styleMedia' in window&&styleMedia===undefined,isSecureContext];
  })()`)),[[1710,1107,1710,1019,0,34,24,24],[800,600,800,652,2,110,34,110,34,0,0,0,0],
    [800,600,0,0,0,0,1],[0,'landscape-primary'],true,6,true,true,true,true,true,true,0,3,4,'undefined','undefined',true,true]);
});

test('Location component writes and window/document assignment use the navigation pipeline',async()=>{
  await pageWith(async page=>{
    assert.equal(await page.evaluate('location.href'),origin+'/done#finish');
    assert.deepEqual(await page.evaluate(`(() => {
      const origins=location.ancestorOrigins;let illegal=false;try{new Location()}catch(e){illegal=e.name==='TypeError'}
      return [document.location===location,location===window.location,String(location)===location.href,location.valueOf()===location,
        location.protocol,location.hostname,location.pathname,location.search,location.hash,location.host===new URL(location.href).host,
        location.origin===new URL(location.href).origin,location.port===new URL(location.href).port,
        Object.hasOwn(location,Symbol.toPrimitive),location[Symbol.toPrimitive]===undefined,
        origins===location.ancestorOrigins,origins.length,origins.item(0),origins.contains('unused'),[...origins],illegal];
    })()`),[true,true,true,true,'http:','127.0.0.1','/done','','#finish',true,true,true,true,true,true,0,null,false,[],true]);
  },'/start');
  await pageWith(page=>page.evaluate('document.title'),'/reload');assert.equal(reloads,2);
});

test('History retains cloned state, traversal and hash events while scroll restoration is local state',async()=>{
  assert.deepEqual(await pageWith(page=>page.evaluate(`(async () => {
    const wait=()=>new Promise(resolve=>setTimeout(resolve,0));const original=location;
    const events=[];window.onpopstate=e=>events.push(['pop',e.state?.step??null]);window.onhashchange=()=>events.push(['hash',location.hash]);
    history.replaceState({step:0},'','#zero');const input={step:1};history.pushState(input,'','#one');input.step=9;
    const copied=history.state.step;history.pushState({step:2},'','#two');history.back();await wait();const back=history.state.step;
    history.forward();await wait();const forward=history.state.step;history.go(-2);await wait();const first=history.state.step;
    history.scrollRestoration='manual';history.scrollRestoration='invalid';
    let error;try{history.pushState({},'','https://other.test/')}catch(e){error=e.name}
    location.hash='final';await wait();const origins=location.ancestorOrigins;
    return [copied,back,forward,first,history.length,history.scrollRestoration,error,location===original,document.location===original,
      location.hash,events,origins.length,document.implementation.createHTMLDocument().location];
  })()`)),[1,1,2,0,2,'manual','SecurityError',true,true,'#final',
    [['pop',1],['hash','#one'],['pop',2],['hash','#two'],['pop',0],['hash','#zero'],['pop',null],['hash','#final']],0,null]);
});
