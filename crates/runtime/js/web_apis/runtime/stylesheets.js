const __sheetToken = {}, __sheetStates = new WeakMap(), __ruleStates = new WeakMap(), __ruleParents = new WeakMap();
const __styleRuleOwners = new WeakMap(), __ownerSheets = new WeakMap(), __adoptedSheets = new WeakMap();
function __sheetState(sheet) { const state=__sheetStates.get(sheet); if (!state) throw new TypeError('Invalid stylesheet'); return state; }
function __ruleState(rule) { const state=__ruleStates.get(rule); if (!state) throw new TypeError('Invalid CSS rule'); return state; }
function __assertSheetReadable(sheet) { if (sheet && !__sheetState(sheet).originClean) throw new DOMException('Stylesheet is not origin-clean','SecurityError'); }
function __setRuleParent(rule, sheet, parent=null) {
    __ruleParents.set(rule,{sheet,parent});
    for (const child of __ruleState(rule).rules ?? []) __setRuleParent(child,sheet,rule);
}
class CSSRuleList {
    constructor(token, items) {
        if (token !== __sheetToken) throw new TypeError('Illegal constructor');
        Object.defineProperty(this,'__items',{value:items,configurable:true});
        return new Proxy(this,{
            get(t,p,r) { if (typeof p==='string' && /^(0|[1-9][0-9]*)$/.test(p)) return t.__items()[Number(p)]; return Reflect.get(t,p,r); },
            ownKeys(t) { return t.__items().map((_,i)=>String(i)); },
            getOwnPropertyDescriptor(t,p) { if (typeof p==='string' && /^(0|[1-9][0-9]*)$/.test(p) && Number(p)<t.__items().length) return {value:t.__items()[Number(p)],enumerable:true,configurable:true}; return Reflect.getOwnPropertyDescriptor(t,p); },
        });
    }
    get length() { return this.__items().length; }
    item(index) { return this.__items()[Number(index)>>>0] ?? null; }
    [Symbol.iterator]() { return this.__items()[Symbol.iterator](); }
}
class MediaList {
    constructor(token,text='') { if(token!==__sheetToken)throw new TypeError('Illegal constructor');this.mediaText=text; }
    get mediaText() { return this.__text; }
    set mediaText(value) { this.__text=String(value).trim(); }
    get length() { return this.__text ? this.__text.split(',').length : 0; }
    item(index) { return this.__text.split(',')[Number(index)]?.trim() || null; }
    appendMedium(value) { value=String(value).trim(); const parts=this.__text?this.__text.split(',').map(s=>s.trim()):[]; if(!parts.includes(value))parts.push(value);this.mediaText=parts.join(', '); }
    deleteMedium(value) { const parts=this.__text.split(',').map(s=>s.trim()), i=parts.indexOf(String(value));if(i<0)throw new DOMException('Medium not found','NotFoundError');parts.splice(i,1);this.mediaText=parts.join(', '); }
    toString() { return this.mediaText; }
}
class StyleSheet {
    constructor(token) { if(token!==__sheetToken)throw new TypeError('Illegal constructor'); }
    get type() { return 'text/css'; }
    get href() { return __sheetState(this).href; }
    get ownerNode() { return __sheetState(this).owner; }
    get parentStyleSheet() { return null; }
    get title() { return this.ownerNode?.getAttribute('title') || null; }
    get media() { return __sheetState(this).media; }
    get disabled() { return __sheetState(this).disabled; }
    set disabled(value) { __sheetState(this).disabled=Boolean(value); }
}
class CSSStyleSheet extends StyleSheet {
    constructor(options={}, token=undefined, owner=null) {
        super(__sheetToken);
        const state={rules:[],list:null,owner,href:owner instanceof HTMLLinkElement?owner.href:null,media:new MediaList(__sheetToken,options.media??''),disabled:!!options.disabled,constructed:token!==__sheetToken,document:owner?.ownerDocument??document,originClean:true,locked:false};
        state.list=new CSSRuleList(__sheetToken,()=>{__assertSheetReadable(this);return state.rules;});__sheetStates.set(this,state);
    }
    get ownerRule() { return null; }
    get cssRules() { __assertSheetReadable(this);return __sheetState(this).list; }
    get rules() { return this.cssRules; }
    insertRule(text,index=0) { return __insertCssRule(this,String(text),index); }
    deleteRule(index) { __deleteCssRule(this,index); }
    replaceSync(text) {
        const s=__sheetState(this);if(!s.constructed||s.locked)throw new DOMException('Cannot replace this stylesheet','NotAllowedError');
        __replaceCssRules(this,String(text));
    }
    replace(text) {
        const s=__sheetState(this);if(!s.constructed||s.locked)return Promise.reject(new DOMException('Cannot replace this stylesheet','NotAllowedError'));
        text=String(text);s.locked=true;
        return Promise.resolve().then(()=>{try{__replaceCssRules(this,text);return this;}finally{s.locked=false;}});
    }
}
class CSSRule {
    constructor(token,state) { if(token!==__sheetToken)throw new TypeError('Illegal constructor');__ruleStates.set(this,state); }
    get type() { return __ruleState(this).kind; }
    get parentStyleSheet() { return __ruleParents.get(this)?.sheet??null; }
    get parentRule() { return __ruleParents.get(this)?.parent??null; }
    get cssText() { return ''; }
    set cssText(_value) {} // CSSOM specifies that setting CSSRule.cssText does nothing.
}
for(const [name,value] of Object.entries({STYLE_RULE:1,IMPORT_RULE:3,MEDIA_RULE:4,FONT_FACE_RULE:5,PAGE_RULE:6,KEYFRAMES_RULE:7,KEYFRAME_RULE:8,NAMESPACE_RULE:10,SUPPORTS_RULE:12})) {
    Object.defineProperty(CSSRule,name,{value});Object.defineProperty(CSSRule.prototype,name,{value});
}
function __ruleStyle(rule,css) {
    // Declaration parsing and storage reuse the existing Lexbor-backed style object.
    const element=document.createElement('span');element.style.cssText=css;
    const style=element.style;__styleRuleOwners.set(__styleDeclarationTarget(style),rule);return style;
}
class CSSStyleRule extends CSSRule {
    constructor(token,state) { super(token,state);state.style=__ruleStyle(this,state.declarations);state.list=new CSSRuleList(__sheetToken,()=>state.rules); }
    get selectorText() { return __ruleState(this).prelude; }
    set selectorText(value) {
        const parsed=JSON.parse(__callHost('cssParseSheet',window,`${String(value)} {}`));
        if(parsed.length===1&&parsed[0].kind===1)__ruleState(this).prelude=parsed[0].prelude;
    }
    get style() { return __ruleState(this).style; }
    get cssRules() { return __ruleState(this).list; }
    insertRule(text,index=0) { return __insertCssRule(this,String(text),index); }
    deleteRule(index) { __deleteCssRule(this,index); }
    get cssText() { const s=__ruleState(this);return `${s.prelude} {${s.style.cssText?' '+s.style.cssText+' ':' '}${s.rules.map(r=>r.cssText).join(' ')}}`; }
}
class CSSGroupingRule extends CSSRule {
    constructor(token,state) { super(token,state);state.list=new CSSRuleList(__sheetToken,()=>state.rules); }
    get cssRules() { return __ruleState(this).list; }
    insertRule(text,index=0) { return __insertCssRule(this,String(text),index); }
    deleteRule(index) { __deleteCssRule(this,index); }
}
class CSSConditionRule extends CSSGroupingRule {
    get conditionText() { return __ruleState(this).prelude; }
}
class CSSMediaRule extends CSSConditionRule {
    constructor(token,state) { super(token,state);state.media=new MediaList(__sheetToken,state.prelude); }
    get media() { return __ruleState(this).media; }
    get conditionText() { return this.media.mediaText; }
    get cssText() { return `@media ${this.conditionText} {\n  ${Array.from(this.cssRules,r=>r.cssText).join('\n  ')}\n}`; }
}
class CSSSupportsRule extends CSSConditionRule {
    get cssText() { return `@supports ${this.conditionText} {\n  ${Array.from(this.cssRules,r=>r.cssText).join('\n  ')}\n}`; }
}
class CSSFontFaceRule extends CSSRule {
    constructor(token,state) { super(token,state);state.style=__ruleStyle(this,state.declarations); }
    get style() { return __ruleState(this).style; }
    get cssText() { return `@font-face { ${this.style.cssText} }`; }
}
function __parseCssRules(text) {
    const records=JSON.parse(__callHost('cssParseSheet',window,text)), roots=[], stack=[];
    for(const record of records) {
        const cls=({1:CSSStyleRule,4:CSSMediaRule,12:CSSSupportsRule,5:CSSFontFaceRule})[record.kind];
        if(!cls)continue;
        const rule=new cls(__sheetToken,{...record,rules:[]});
        if(record.depth===0)roots.push(rule);else if(stack[record.depth-1])__ruleState(stack[record.depth-1]).rules.push(rule);
        stack[record.depth]=rule;stack.length=record.depth+1;
    }
    return roots;
}
function __replaceCssRules(sheet,text) {
    const s=__sheetState(sheet), rules=__parseCssRules(text);
    for(const rule of s.rules)__setRuleParent(rule,null);
    s.rules=rules;for(const rule of rules)__setRuleParent(rule,sheet);
}
function __insertCssRule(container,text,index) {
    const sheet=container instanceof CSSStyleSheet?container:container.parentStyleSheet;
    __assertSheetReadable(sheet);if(sheet&&__sheetState(sheet).locked)throw new DOMException('Stylesheet is locked','NotAllowedError');
    const rules=container instanceof CSSStyleSheet?__sheetState(container).rules:__ruleState(container).rules;
    index=Number(index)>>>0;if(index>rules.length)throw new DOMException('Index is outside the rule list','IndexSizeError');
    const parsed=__parseCssRules(text);if(parsed.length!==1)throw new DOMException('Expected a single CSS rule','SyntaxError');
    const rule=parsed[0];rules.splice(index,0,rule);__setRuleParent(rule,sheet,container instanceof CSSStyleSheet?null:container);return index;
}
function __deleteCssRule(container,index) {
    const sheet=container instanceof CSSStyleSheet?container:container.parentStyleSheet;
    __assertSheetReadable(sheet);if(sheet&&__sheetState(sheet).locked)throw new DOMException('Stylesheet is locked','NotAllowedError');
    const rules=container instanceof CSSStyleSheet?__sheetState(container).rules:__ruleState(container).rules;
    index=Number(index)>>>0;if(index>=rules.length)throw new DOMException('Index is outside the rule list','IndexSizeError');
    __setRuleParent(rules.splice(index,1)[0],null);
}
class StyleSheetList extends CSSRuleList {}
function __sheetForOwner(owner) {
    if(!owner.isConnected || owner.type && owner.type.toLowerCase()!=='text/css') { __ownerSheets.delete(owner);return null; }
    if(owner instanceof HTMLLinkElement && !owner.relList.contains('stylesheet')) { __ownerSheets.delete(owner);return null; }
    const source=owner instanceof HTMLStyleElement?owner.textContent:owner.href;
    let entry=__ownerSheets.get(owner);
    if(entry?.source===source) {
        if(entry.sheet && entry.media!==owner.media) { entry.media=owner.media;entry.sheet.media.mediaText=owner.media; }
        return entry.sheet;
    }
    const sheet=new CSSStyleSheet({media:owner.media},__sheetToken,owner);
    entry={source,media:owner.media,sheet:owner instanceof HTMLStyleElement?sheet:null};__ownerSheets.set(owner,entry);
    if(owner instanceof HTMLStyleElement)__replaceCssRules(sheet,source);
    else if(source) {
        fetch(source).then(async response=>{
            if(!response.ok)throw new Error('Stylesheet request failed');
            const css=await response.text();if(__ownerSheets.get(owner)!==entry||!owner.isConnected)return;
            __replaceCssRules(sheet,css);__sheetState(sheet).href=response.url||source;__sheetState(sheet).originClean=new URL(response.url||source,document.URL).origin===new URL(document.URL).origin;
            entry.sheet=sheet;owner.dispatchEvent(new Event('load'));
        }).catch(()=>{if(__ownerSheets.get(owner)===entry)owner.dispatchEvent(new Event('error'));});
    }
    return entry.sheet;
}
for(const cls of [HTMLStyleElement,HTMLLinkElement])Object.defineProperty(cls.prototype,'sheet',{configurable:true,get(){return __sheetForOwner(this);}});
for(const cls of [Document,ShadowRoot])Object.defineProperties(cls.prototype,{
    styleSheets:{configurable:true,get(){return __cachedCollection(this,'styleSheets',()=>new StyleSheetList(__sheetToken,()=>Array.from(this.querySelectorAll('style,link')).map(__sheetForOwner).filter(Boolean)));}},
    adoptedStyleSheets:{configurable:true,get(){
        if(!__adoptedSheets.has(this)) {
            const owner=this instanceof Document?this:this.ownerDocument;
            const validate=sheet=>{if(!(sheet instanceof CSSStyleSheet)||!__sheetState(sheet).constructed||__sheetState(sheet).document!==owner)throw new DOMException('Cannot adopt this stylesheet','NotAllowedError');};
            const array=new Proxy([],{set(t,p,v){if(/^(0|[1-9][0-9]*)$/.test(String(p)))validate(v);return Reflect.set(t,p,v);}});
            __adoptedSheets.set(this,{array,validate});
        }
        return __adoptedSheets.get(this).array;
    },set(value){const values=Array.from(value);this.adoptedStyleSheets;const {array,validate}=__adoptedSheets.get(this);values.forEach(validate);array.splice(0,array.length,...values);}},
});
let __updatingStyleSheets = false;
function __updateStyleSheets() {
    if (__updatingStyleSheets) return;
    __updatingStyleSheets=true;
    try { for(const owner of document.querySelectorAll('style,link[rel~=stylesheet]'))__sheetForOwner(owner); }
    finally { __updatingStyleSheets=false; }
}

function __styleMutationBefore(operation, args) {
    const owners = new Set();
    const add = node => {
        if (node instanceof HTMLStyleElement || node instanceof HTMLLinkElement) owners.add(node);
        if (node?.parentNode instanceof HTMLStyleElement) owners.add(node.parentNode);
    };
    const subtree = node => {
        add(node);
        if (node instanceof Element || node instanceof DocumentFragment) for (const owner of node.querySelectorAll('style,link')) owners.add(owner);
    };
    const [target,child,reference] = args;
    add(target);
    if (['appendChild','insertBefore','removeChild','replaceChild'].includes(operation)) { subtree(child); if(operation==='replaceChild')subtree(reference); }
    if (['setTextContent','setInnerHTML'].includes(operation)) subtree(target);
    return () => {
        if (['setTextContent','setInnerHTML'].includes(operation)) subtree(target);
        for (const owner of owners) __sheetForOwner(owner);
    };
}
