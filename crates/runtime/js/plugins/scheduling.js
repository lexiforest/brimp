({global}) => {
    const priorities=['user-blocking','user-visible','background'];
    const priority=value=>{if(!priorities.includes(value))throw new TypeError('Invalid task priority');return value;};
    const signalPriorities=new WeakMap();
    class TaskPriorityChangeEvent extends Event {
        constructor(type,options={}){super(type,options);this.previousPriority=priority(options.previousPriority);}
    }
    class TaskSignal extends AbortSignal {
        get priority(){return signalPriorities.get(this);}
    }
    class TaskController extends AbortController {
        constructor(options={}){super();Object.setPrototypeOf(this.signal,TaskSignal.prototype);signalPriorities.set(this.signal,priority(options.priority??'user-visible'));}
        setPriority(value){value=priority(value);const previousPriority=this.signal.priority;if(value===previousPriority)return;signalPriorities.set(this.signal,value);this.signal.dispatchEvent(new TaskPriorityChangeEvent('prioritychange',{previousPriority}));}
    }
    const queue=[];let timer=null, nextId=0, closed=false;
    function arm(){if(timer!==null)clearTimeout(timer);timer=null;if(queue.length)timer=setTimeout(run,Math.max(0,Math.min(...queue.map(t=>t.due))-performance.now()));}
    function remove(task){const i=queue.indexOf(task);if(i>=0)queue.splice(i,1);task.signal?.removeEventListener('abort',task.abort);}
    function run(){timer=null;const now=performance.now();const due=queue.filter(t=>t.due<=now).sort((a,b)=>priorities.indexOf(a.priority??a.signal?.priority??'user-visible')-priorities.indexOf(b.priority??b.signal?.priority??'user-visible')||a.id-b.id);const task=due[0];if(task){remove(task);try{task.resolve(task.callback());}catch(e){task.reject(e);}}arm();}
    class Scheduler {
        postTask(callback,options={}) {
            return new Promise((resolve,reject)=>{
                if(closed)throw new DOMException('Scheduler closed','AbortError');
                if(typeof callback!=='function')throw new TypeError('Callback required');
                const signal=options.signal;if(signal && !(signal instanceof AbortSignal))throw new TypeError('Invalid signal');
                if(signal?.aborted){reject(signal.reason);return;}
                const delay=Number(options.delay??0);if(!Number.isFinite(delay)||delay<0)throw new TypeError('Invalid delay');
                const task={callback,resolve,reject,signal,priority:options.priority===undefined?undefined:priority(options.priority),due:performance.now()+delay,id:nextId++};
                task.abort=()=>{remove(task);reject(signal.reason);arm();};signal?.addEventListener('abort',task.abort,{once:true});queue.push(task);arm();
            });
        }
        yield(){return this.postTask(()=>undefined);}
    }
    const idle=new Set();
    function requestIdleCallback(callback,options={}) {
        if(typeof callback!=='function')throw new TypeError('Callback required');
        const timeout=options.timeout===undefined?Infinity:Number(options.timeout);if(Number.isNaN(timeout)||timeout<0)throw new TypeError('Invalid timeout');
        const start=performance.now();const id=setTimeout(()=>{idle.delete(id);callback({didTimeout:performance.now()-start>=timeout,timeRemaining:()=>0});},0);idle.add(id);return id;
    }
    function cancelIdleCallback(id){idle.delete(id);clearTimeout(id);}
    for(const [name,value] of Object.entries({Scheduler,TaskController,TaskSignal,TaskPriorityChangeEvent,requestIdleCallback,cancelIdleCallback,scheduler:new Scheduler()}))Object.defineProperty(global,name,{value,writable:true,configurable:true});
    return ()=>{closed=true;if(timer!==null)clearTimeout(timer);for(const task of [...queue]){remove(task);task.reject(new DOMException('Realm closed','AbortError'));}for(const id of idle)clearTimeout(id);idle.clear();};
}
