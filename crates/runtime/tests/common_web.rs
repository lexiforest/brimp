use brimp_runtime::runtime::{
    clock::{ClockOptions, TimeMode},
    native::{Session, SessionOptions},
};
use serde_json::json;

fn session() -> Session {
    Session::new(SessionOptions {
        clock: ClockOptions {
            mode: TimeMode::Logical,
            epoch_ms: Some(0),
        },
        ..Default::default()
    })
    .unwrap()
}
#[test]
fn ranges_extract_partial_trees_and_keep_boundaries_live() {
    let session = session();
    let page = session.new_page().unwrap();
    assert_eq!(page.evaluate(r#"(()=>{
      document.body.innerHTML='<p>hello <b>world</b>!</p>';
      const p=document.querySelector('p'),r=document.createRange();r.selectNodeContents(p);
      if(r.toString()!=='hello world!') throw Error('range text');
      r.setStart(p.firstChild,2);r.setEnd(p.querySelector('b').firstChild,3);
      const clone=r.cloneContents();if(clone.textContent!=='llo wor'||p.textContent!=='hello world!')throw Error('clone');
      const extracted=r.extractContents();if(extracted.textContent!=='llo wor'||p.innerHTML!=='he<b>ld</b>!')throw Error('extract: '+p.innerHTML);
      if(!r.collapsed)throw Error('collapse');
      const live=document.createRange();live.selectNodeContents(p.querySelector('b').firstChild);
      p.querySelector('b').firstChild.insertData(1,'XYZ');if(live.toString()!=='lXYZd')throw Error('live data');
      const b=p.querySelector('b');b.remove();if(live.startContainer!==p||!live.collapsed)throw Error('live removal');
      const text=p.firstChild,split=document.createRange();split.setStart(text,1);split.setEnd(text,2);
      const tail=text.splitText(1);if(split.endContainer!==tail||split.toString()!=='e')throw Error('split');
      const merged=document.createElement('p');const t1=document.createTextNode('ab'),t2=document.createTextNode('cd');merged.append(t1,t2);
      const normalized=document.createRange();normalized.setStart(t2,1);normalized.setEnd(t2,2);merged.normalize();if(normalized.startContainer!==t1||normalized.startOffset!==3||normalized.toString()!=='d')throw Error('normalize');
      const table=document.createElement('table');const ctx=document.createRange();ctx.selectNodeContents(table);
      const f=ctx.createContextualFragment('<tr><td>x</td></tr>');if(f.firstChild.localName!=='tbody')throw Error('context');
      const ins=document.createRange();ins.setStart(tail,1);ins.collapse(true);ins.insertNode(document.createElement('i'));if(ins.collapsed)throw Error('insert');
      return true;
    })()"#).unwrap(),json!(true));
}
#[test]
fn file_reader_is_async_abortable_and_reentrant() {
    let session = session();
    let page = session.new_page().unwrap();
    assert_eq!(page.evaluate(r#"(async()=>{
      const r=new FileReader(),events=[];for(const t of ['loadstart','progress','load','loadend'])r.addEventListener(t,e=>events.push([t,e.isTrusted]));
      const done=new Promise(resolve=>r.onloadend=resolve);r.readAsText(new Blob(['hello']));
      if(r.readyState!==1||r.result!==null)throw Error('sync');
      let rejected=false;try{r.readAsText(new Blob([]))}catch(e){rejected=e.name==='InvalidStateError'}
      await done;if(!rejected||r.result!=='hello'||r.readyState!==2||!events.every(e=>e[1]))throw Error('read');
      const aborted=[];r.onabort=()=>aborted.push('abort');r.onloadend=()=>aborted.push('end');r.readAsText(new Blob(['x']));r.abort();
      if(aborted.join()!=='abort,end'||r.result!==null)throw Error('abort');
      const binary=new Promise(resolve=>r.onloadend=resolve);r.readAsDataURL(new Blob([new Uint8Array([0,255])],{type:'image/test'}));await binary;
      if(r.result!=='data:image/test;base64,AP8=')throw Error('data url');
      return events.slice(0,4).map(e=>e[0]);
    })()"#).unwrap(),json!(["loadstart","progress","load","loadend"]));
}
#[test]
fn streams_preserve_backpressure_closure_and_cancellation() {
    let session = session();
    let page = session.new_page().unwrap();
    assert_eq!(page.evaluate(r#"(async()=>{
      let closed=false,finish;const w=new WritableStream({write(){return new Promise(r=>finish=r)}}).getWriter();w.closed.then(()=>closed=true);
      await Promise.resolve();if(closed)throw Error('premature closed');
      const write=w.write('x');await Promise.resolve();if(w.desiredSize!==0)throw Error('backpressure');finish();await write;await w.close();await w.closed;
      const transform=new TransformStream({transform(x,c){c.enqueue(x*2)}}),writer=transform.writable.getWriter(),reader=transform.readable.getReader();
      const writing=writer.write(21);const item=await reader.read();await writing;await writer.close();if(item.value!==42||!(await reader.read()).done)throw Error('transform');
      let reason;const stream=new ReadableStream({pull(c){c.enqueue(1)},cancel(r){reason=r}});const sr=stream.getReader();await sr.read();await sr.cancel('done');if(reason!=='done')throw Error('cancel');
      const failure=new Error('sink');const bad=new WritableStream({write(){throw failure}}).getWriter();bad.closed.catch(()=>{});
      try{await bad.write(1);return false}catch(e){return e===failure&&closed}
    })()"#).unwrap(),json!(true));
}
