export const inputFileCases=['text','image','file','FILE','number','hidden'].flatMap(type=>
 ['null','undefined','string','object','array','fake','proxy','file','blob','symbol','poison','filelist','empty'].map(kind=>({type,kind})));
export function measureInputFiles(cases){return cases.map(({type,kind})=>{
 const i=document.createElement('input');i.type=type;const dt=new DataTransfer();dt.items.add(new File(['data'],'test.txt'));
 const list=dt.files,before=new DataTransfer();before.items.add(new File(['before'],'before.txt'));i.files=before.files;
 let converted=0;const values={null:null,undefined:undefined,string:'',object:{},array:[],fake:Object.create(FileList.prototype),proxy:new Proxy(list,{}),file:list[0],blob:new Blob(['x']),symbol:Symbol(),poison:{[Symbol.toPrimitive](){converted++;throw Error('coercion')}},filelist:list,empty:new DataTransfer().files};
 const events=[];for(const name of ['input','change','invalid'])i.addEventListener(name,()=>events.push(name));
 const observer=new MutationObserver(()=>{});observer.observe(i,{attributes:true,childList:true,subtree:true});
 let error=null;try{i.files=values[kind]}catch(e){error=e.name}
 const result=[error,converted,i.files===null?null:Array.from(i.files,f=>f.name),i.files===list,events,observer.takeRecords().length];observer.disconnect();return result;
});}
