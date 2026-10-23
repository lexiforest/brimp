const __customValidity = new WeakMap(), __validityStates = new WeakMap();
const __formControlSelector = 'button,fieldset,input,object,output,select,textarea';
function __formControls(form) {
    return Array.from(form.getRootNode().querySelectorAll(__formControlSelector)).filter(e => __formOwner(e) === form && !(e instanceof HTMLInputElement && e.type === 'image'));
}
function __labelable(e) {
    return e instanceof HTMLElement && ['button','input','meter','output','progress','select','textarea'].includes(e.localName) && !(e instanceof HTMLInputElement && e.type === 'hidden');
}
function __labelsFor(e) {
    return __cachedCollection(e, 'labels', () => new NodeList(() => Array.from(e.getRootNode().querySelectorAll('label')).filter(l => l.control === e)));
}
Object.defineProperty(HTMLLabelElement.prototype, 'control', {configurable:true, get() {
    if (this.hasAttribute('for')) {
        const root = this.getRootNode();
        const e = root instanceof Element ? [root,...root.querySelectorAll('[id]')].find(e=>e.id===this.htmlFor) : root.getElementById(this.htmlFor);
        return __labelable(e) ? e : null;
    }
    return Array.from(this.querySelectorAll('*')).find(__labelable) ?? null;
}});
Object.defineProperty(HTMLLabelElement.prototype, 'form', {configurable:true, get() { return this.control?.form ?? null; }});
for (const cls of [HTMLInputElement,HTMLButtonElement,HTMLSelectElement,HTMLTextAreaElement,HTMLFieldSetElement,HTMLObjectElement,HTMLOutputElement]) {
    Object.defineProperty(cls.prototype, 'form', {configurable:true, get() { return __formOwner(this); }});
}
for (const cls of [HTMLInputElement,HTMLButtonElement,HTMLSelectElement,HTMLTextAreaElement,HTMLMeterElement,HTMLProgressElement,HTMLOutputElement]) {
    Object.defineProperty(cls.prototype, 'labels', {configurable:true, get() { return __labelable(this) ? __labelsFor(this) : null; }});
}
Object.defineProperty(HTMLFormElement.prototype, 'elements', {configurable:true, get() {
    return __cachedCollection(this, 'elements', () => new HTMLFormControlsCollection(() => __formControls(this)));
}});
const __resettingForms = new WeakSet();
HTMLFormElement.prototype.reset = function() {
    if (__resettingForms.has(this)) return;
    __resettingForms.add(this);
    try {
        if (!this.dispatchEvent(new Event('reset', {bubbles:true,cancelable:true}))) return;
        for (const e of __formControls(this)) {
            __inputValues.delete(e); __inputCheckedStates.delete(e); __controlSelections.delete(e);
            if (e instanceof HTMLInputElement && e.type === 'file') e.__setFiles([]);
            if (e instanceof HTMLSelectElement) for (const option of e.querySelectorAll('option')) __optionSelectedStates.delete(option);
        }
    } finally { __resettingForms.delete(this); }
};
function __numericValue(value) {
    return /^-?(?:\d+(?:\.\d+)?|\.\d+)(?:[eE][+-]?\d+)?$/.test(value) && Number.isFinite(Number(value)) ? Number(value) : NaN;
}
function __numberParameters(e) {
    const min = __numericValue(e.min), max = __numericValue(e.max), attr = __numericValue(e.getAttribute('value') ?? '');
    let step = e.step === 'any' ? NaN : __numericValue(e.step);
    if (e.step !== 'any' && !(step > 0)) step = 1;
    return {min,max,step,base:Number.isFinite(min) ? min : Number.isFinite(attr) ? attr : 0};
}
function __stepMismatch(value, base, step) {
    if (!Number.isFinite(value) || !Number.isFinite(step)) return false;
    const q = (value-base)/step;
    return Math.abs(q-Math.round(q)) > 1e-7;
}
function __willValidate(e) {
    return !__isDisabledFormControl(e) && !e.closest('datalist') &&
        !(e instanceof HTMLInputElement && ['hidden','reset','button'].includes(e.type)) &&
        !(e instanceof HTMLButtonElement && e.type !== 'submit') &&
        ![HTMLFieldSetElement,HTMLOutputElement,HTMLObjectElement].some(c => e instanceof c) &&
        !(e.readOnly && (e instanceof HTMLInputElement || e instanceof HTMLTextAreaElement));
}
function __validityFlags(e) {
    const flags = {valueMissing:false,typeMismatch:false,patternMismatch:false,tooLong:false,tooShort:false,rangeUnderflow:false,rangeOverflow:false,stepMismatch:false,badInput:false,customError:!!__customValidity.get(e)};
    const value = e.value ?? '';
    if (e.required) {
        if (e instanceof HTMLInputElement && e.type === 'checkbox') flags.valueMissing = !e.checked;
        else if (e instanceof HTMLInputElement && e.type === 'radio') {
            const group = Array.from(e.getRootNode().querySelectorAll('input')).filter(n => n.type === 'radio' && n.name === e.name && n.form === e.form);
            flags.valueMissing = !group.some(n => n.checked);
        } else if (e instanceof HTMLInputElement && e.type === 'file') flags.valueMissing = !e.files.length;
        else flags.valueMissing = value === '';
    }
    if (e instanceof HTMLInputElement && value !== '') {
        if (e.type === 'email') flags.typeMismatch = !(e.multiple ? value.split(',').map(s=>s.trim()) : [value]).every(s => /^[a-zA-Z0-9.!#$%&'*+\/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]*[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]*[a-zA-Z0-9])?)*$/.test(s));
        if (e.type === 'url') { try { new URL(value); } catch { flags.typeMismatch = true; } }
        if (['text','search','tel','url','email','password'].includes(e.type) && e.hasAttribute('pattern')) {
            try { flags.patternMismatch = !new RegExp(`^(?:${e.pattern})$`, 'v').test(value); } catch { /* Invalid patterns impose no constraint. */ }
        }
        if (['number','range'].includes(e.type)) {
            const n = __numericValue(value), {min,max,base,step} = __numberParameters(e);
            flags.rangeUnderflow = Number.isFinite(min) && n < min;
            flags.rangeOverflow = Number.isFinite(max) && n > max;
            flags.stepMismatch = __stepMismatch(n,base,step);
        }
    }
    return flags;
}
const __validityToken = {};
class ValidityState {
    #element;
    constructor(token, element) { if (token !== __validityToken) throw new TypeError('Illegal constructor'); this.#element = element; }
    get valid() { return !Object.values(__validityFlags(this.#element)).some(Boolean); }
    __flag(name) { return __validityFlags(this.#element)[name]; }
}
for (const flag of ['valueMissing','typeMismatch','patternMismatch','tooLong','tooShort','rangeUnderflow','rangeOverflow','stepMismatch','badInput','customError']) {
    Object.defineProperty(ValidityState.prototype, flag, {configurable:true, get() { return this.__flag(flag); }});
}
function __getValidity(e) {
    if (!__validityStates.has(e)) __validityStates.set(e, new ValidityState(__validityToken, e));
    return __validityStates.get(e);
}
function __checkValidity(e) {
    if (!e.willValidate || e.validity.valid) return true;
    e.dispatchEvent(new Event('invalid', {cancelable:true})); return false;
}
for (const cls of [HTMLInputElement,HTMLButtonElement,HTMLSelectElement,HTMLTextAreaElement,HTMLFieldSetElement,HTMLObjectElement,HTMLOutputElement]) {
    Object.defineProperties(cls.prototype, {
        validity:{configurable:true,get() { return __getValidity(this); }},
        willValidate:{configurable:true,get() { return __willValidate(this); }},
        validationMessage:{configurable:true,get() { return !this.willValidate || this.validity.valid ? '' : __customValidity.get(this) || 'The value does not satisfy the constraints.'; }},
    });
    cls.prototype.setCustomValidity = function(message) { __customValidity.set(this, String(message)); };
    cls.prototype.checkValidity = function() { return __checkValidity(this); };
    cls.prototype.reportValidity = function() { return __checkValidity(this); };
}
HTMLFormElement.prototype.checkValidity = function() {
    let valid = true; for (const e of this.elements) if (typeof e.checkValidity === 'function' && !e.checkValidity()) valid = false; return valid;
};
HTMLFormElement.prototype.reportValidity = function() { return this.checkValidity(); };
Object.defineProperty(HTMLInputElement.prototype, 'valueAsNumber', {configurable:true,
    get() { return ['number','range'].includes(this.type) ? __numericValue(this.value) : NaN; },
    set(value) { if (!['number','range'].includes(this.type)) throw new DOMException('Not a numeric control','InvalidStateError'); value=Number(value); if (!Number.isFinite(value) && !Number.isNaN(value)) throw new TypeError('Value must be finite'); this.value=Number.isNaN(value)?'':String(value); },
});
function __stepInput(e, count, direction) {
    if (!['number','range'].includes(e.type)) throw new DOMException('This control cannot be stepped','InvalidStateError');
    const {min,max,base,step} = __numberParameters(e);
    if (!Number.isFinite(step)) throw new DOMException('No allowed step','InvalidStateError');
    count = Number(count) >> 0;
    if (Number.isFinite(min) && Number.isFinite(max) && min > max) return;
    const old = Number.isFinite(e.valueAsNumber) ? e.valueAsNumber : 0;
    let n = old, delta = count * direction;
    if (__stepMismatch(n,base,step)) {
        n = base + (delta < 0 ? Math.floor((n-base)/step) : Math.ceil((n-base)/step))*step;
        delta -= Math.sign(delta);
    }
    n += delta*step;
    if (Number.isFinite(min) && n < min) n = base+Math.ceil((min-base)/step)*step;
    if (Number.isFinite(max) && n > max) n = base+Math.floor((max-base)/step)*step;
    if (count*direction > 0 && n < old || count*direction < 0 && n > old) return;
    e.valueAsNumber = Number(n.toPrecision(15));
}
HTMLInputElement.prototype.stepUp = function(n=1) { __stepInput(this,n,1); };
HTMLInputElement.prototype.stepDown = function(n=1) { __stepInput(this,n,-1); };
function __supportsSelection(e) { return e instanceof HTMLTextAreaElement || ['text','search','url','tel','password'].includes(e.type); }
for (const cls of [HTMLInputElement,HTMLTextAreaElement]) {
    cls.prototype.setRangeText = function(replacement, start, end, mode='preserve') {
        if (!__supportsSelection(this)) throw new DOMException('Selection is unavailable','InvalidStateError');
        if (arguments.length < 1 || arguments.length === 2) throw new TypeError('Expected one or at least three arguments');
        replacement = String(replacement); mode = String(mode);
        if (!['select','start','end','preserve'].includes(mode)) throw new TypeError('Invalid selection mode');
        const selection = __controlSelection(this), old = this.value;
        if (arguments.length === 1) { start=selection.start; end=selection.end; }
        start=Number(start)>>>0; end=Number(end)>>>0;
        if (start>end) throw new DOMException('Start is after end','IndexSizeError');
        start=Math.min(start,old.length);end=Math.min(end,old.length);
        this.value=old.slice(0,start)+replacement+old.slice(end);
        const last=start+replacement.length, delta=replacement.length-(end-start);
        let a=start,b=last;
        if (mode==='start') b=a; else if (mode==='end') a=b;
        else if (mode==='preserve') {
            a=selection.start>end?selection.start+delta:selection.start>start?start:selection.start;
            b=selection.end>end?selection.end+delta:selection.end>start?last:selection.end;
        }
        this.setSelectionRange(a,b);
    };
}
