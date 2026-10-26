const __controlSelections = new WeakMap();
const __selectionTypes = ['text','search','url','tel','password'];
const __selectionChangeQueued = new WeakSet();
const __focusTextControl = Element.prototype.focus;
function __supportsSelection(control) {
    return control instanceof HTMLTextAreaElement || __selectionTypes.includes(__inputType(control));
}
function __controlValue(control) {
    return control instanceof HTMLInputElement ? __inputValue(control)
        : (__inputValues.has(control) ? __inputValues.get(control) : __textareaDefaultValue(control)).replace(/\r\n?/g,'\n');
}
function __controlConnected(control) {
    let root=control,parent;
    while ((parent=__callHost('parentNode',root) ?? __shadowData.get(root)?.host)!=null) root=parent;
    return root instanceof Document;
}
function __controlFocused(control) { return __activeElement===control && __controlConnected(control); }
function __controlSelection(control) {
    // Safari retains cached offsets when an unfocused input's default value
    // shrinks. Clamp only when a selection API explicitly updates the range.
    return {...(__controlSelections.get(control) ?? {start:0,end:0,direction:'none'})};
}
function __controlValueChanged(control, previous, value) {
    if (previous!==value) __controlSelections.set(control,{start:value.length,end:value.length,direction:'none'});
}
function __selectionMutationBefore(operation,args) {
    if (!['setTextContent','appendChild','removeChild','insertBefore','replaceChild','setInnerHTML'].includes(operation)) return null;
    const controls=new Set();
    for(const [index,argument] of args.entries()) {
        if(!__callHost('isNativeNode', argument)) continue;
        // Moving a textarea does not change its own child text. For inserted
        // or removed nodes, only their former ancestors need an update.
        for(let node=index===0?argument:__callHost('parentNode',argument);node;node=__callHost('parentNode',node)) {
            if(node instanceof HTMLTextAreaElement && !__inputValues.has(node)) controls.add(node);
        }
    }
    if(!controls.size) return null;
    return () => {
        for(const control of controls) {
            __controlSelections.set(control,{start:0,end:0,direction:__controlFocused(control)?'forward':'none'});
        }
    };
}
function __setControlSelection(control,start,end,direction,notify=true) {
    const length=(control instanceof HTMLInputElement && !__supportsSelection(control) ? __inputEditorValue(control) : __controlValue(control)).length, previous=__controlSelection(control);
    end=Math.min(end,length);start=Math.min(start,end);
    if(direction!=='forward' && direction!=='backward') direction='none';
    if(start===end && direction==='backward' && __controlFocused(control)) direction='forward';
    __controlSelections.set(control,{start,end,direction});
    if(!notify || previous.start===start && previous.end===end && previous.direction===direction) return;
    setTimeout(()=>Reflect.apply(__dispatchEvent,control,[__markTrustedEvent(new Event('select',{bubbles:true}))]),0);
    if(!__selectionChangeQueued.has(control)) {
        __selectionChangeQueued.add(control);
        setTimeout(()=>{
            __selectionChangeQueued.delete(control);
            Reflect.apply(__dispatchEvent,control,[__markTrustedEvent(new Event('selectionchange',{bubbles:true}))]);
        },0);
    }
}
function __requireControlSelection(control) {
    if(!__supportsSelection(control)) throw new DOMException('Selection is unavailable','InvalidStateError');
}
for(const prototype of [HTMLInputElement.prototype,HTMLTextAreaElement.prototype]) {
    const members={
        get selectionStart(){return __supportsSelection(this)?__controlSelection(this).start:null;},
        set selectionStart(value){
            value=(+value)>>>0;__requireControlSelection(this);const s=__controlSelection(this);
            __setControlSelection(this,value,Math.max(value,s.end),s.direction);
        },
        get selectionEnd(){return __supportsSelection(this)?__controlSelection(this).end:null;},
        set selectionEnd(value){
            value=(+value)>>>0;__requireControlSelection(this);const s=__controlSelection(this);
            __setControlSelection(this,s.start,value,s.direction);
        },
        get selectionDirection(){return __supportsSelection(this)?__controlSelection(this).direction:null;},
        set selectionDirection(value){
            value=value===null?'none':`${value}`;__requireControlSelection(this);const s=__controlSelection(this);
            __setControlSelection(this,s.start,s.end,value);
        },
        setSelectionRange(start,end,direction='none'){
            if(arguments.length<2) throw new TypeError('Expected two arguments');
            start=(+start)>>>0;end=(+end)>>>0;direction=direction===null?'none':`${direction}`;
            __requireControlSelection(this);__setControlSelection(this,start,end,direction);
        },
        select(){
            if(this instanceof HTMLInputElement && ![...__selectionTypes,'email','number'].includes(__inputType(this))) return;
            __setControlSelection(this,0,(this instanceof HTMLInputElement ? __inputEditorValue(this) : __controlValue(this)).length,'none');
            if(__controlConnected(this) && !__isDisabledFormControl(this)) Reflect.apply(__focusTextControl,this,[]);
        },
        setRangeText(replacement,start,end,mode='preserve'){
            if(arguments.length<1 || arguments.length===2) throw new TypeError('Expected one or at least three arguments');
            replacement=`${replacement}`;
            if(arguments.length>=3){start=(+start)>>>0;end=(+end)>>>0;mode=__asciiLowercase(`${mode}`);}
            __requireControlSelection(this);
            const selection=__controlSelection(this),old=__controlValue(this);
            if(arguments.length===1){start=selection.start;end=selection.end;}
            if(start>end) throw new DOMException('Start is after end','IndexSizeError');
            start=Math.min(start,old.length);end=Math.min(end,old.length);
            const value=old.slice(0,start)+replacement+old.slice(end);
            const sanitized=this instanceof HTMLInputElement?__sanitizeInputValue(this,__inputType(this),value):value.replace(/\r\n?/g,'\n');
            if(this instanceof HTMLInputElement) __inputScriptValue(this,old,sanitized);
            __inputValues.set(this,sanitized);
            const last=start+replacement.length,delta=replacement.length-(end-start);
            let a=start,b=last;
            if(mode==='start')b=a;else if(mode==='end')a=b;else if(mode!=='select') {
                a=selection.start>end?selection.start+delta:selection.start>start?start:selection.start;
                b=selection.end>end?selection.end+delta:selection.end>start?last:selection.end;
            }
            __setControlSelection(this,a,b,'none');
        },
    };
    Object.defineProperty(members.setRangeText,'length',{value:1});
    for(const name of Object.keys(members)) Object.defineProperty(prototype,name,Object.getOwnPropertyDescriptor(members,name));
}
