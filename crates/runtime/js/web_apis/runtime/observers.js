class MutationRecord {
    constructor(type, target) {
        this.type = type;
        this.target = target;
        this.addedNodes = __createNodeList([]);
        this.removedNodes = __createNodeList([]);
        this.previousSibling = null;
        this.nextSibling = null;
        this.attributeName = null;
        this.attributeNamespace = null;
        this.oldValue = null;
    }
}
const __mutationObservers = new Set();
let __mutationDeliveryQueued = false;
class MutationObserver {
    constructor(callback) {
        if (typeof callback !== 'function') throw new TypeError('MutationObserver callback must be callable');
        this.__callback = callback;
        this.__targets = new Map();
        this.__records = [];
        this.__transient = new Map();
    }
    observe(target, options = {}) {
        if (!(target instanceof Node)) throw new TypeError('MutationObserver target must be a Node');
        const opts = { ...options };
        if (opts.attributeOldValue || opts.attributeFilter !== undefined) {
            if (opts.attributes === false) throw new TypeError('attribute options require attributes');
            opts.attributes = true;
        }
        if (opts.characterDataOldValue) {
            if (opts.characterData === false) throw new TypeError('old character data requires characterData');
            opts.characterData = true;
        }
        if (!opts.childList && !opts.attributes && !opts.characterData) throw new TypeError('at least one mutation type is required');
        if (opts.attributeFilter !== undefined) opts.attributeFilter = Array.from(opts.attributeFilter, String);
        this.__targets.set(target, opts);
        __mutationObservers.add(this);
    }
    disconnect() {
        this.__targets.clear(); this.__transient.clear(); this.__records = [];
        __mutationObservers.delete(this);
    }
    takeRecords() { return this.__records.splice(0); }
}
function __mutationAncestors(target) {
    const ancestors = new Set();
    for (let node = target; node; node = __callHost('parentNode', node)) ancestors.add(node);
    return ancestors;
}
function __queueMutation(record, ancestors = __mutationAncestors(record.target)) {
    if (__callHost('mutationObserved')) {
        const id = node => __callHost('mutationNodeId', node);
        __callHost('mutationEvent', globalThis, JSON.stringify({kind: record.type, target: id(record.target), added: Array.from(record.addedNodes, id), removed: Array.from(record.removedNodes, id), attribute: record.attributeName, oldValue: record.oldValue}));
    }
    for (const observer of __mutationObservers) {
        let interested = false, oldValue = false;
        for (const [root, options] of [...observer.__targets, ...observer.__transient]) {
            if (root !== record.target && !(options.subtree && ancestors.has(root))) continue;
            if (record.type === 'childList' && options.subtree) {
                for (const removed of record.removedNodes) observer.__transient.set(removed, options);
            }
            if (!options[record.type]) continue;
            if (record.type === 'attributes' && options.attributeFilter && !options.attributeFilter.includes(record.attributeName)) continue;
            interested = true;
            oldValue ||= record.type === 'attributes' ? !!options.attributeOldValue : !!options.characterDataOldValue;
        }
        if (interested) {
            const copy = Object.assign(new MutationRecord(record.type, record.target), record);
            if (!oldValue) copy.oldValue = null;
            observer.__records.push(copy);
        }
    }
    if (!__mutationDeliveryQueued) {
        __mutationDeliveryQueued = true;
        queueMicrotask(() => {
            __mutationDeliveryQueued = false;
            const deliveries = [...__mutationObservers].map(observer => [observer, observer.takeRecords()]);
            for (const [observer, records] of deliveries) {
                observer.__transient.clear();
                if (records.length) {
                    try { observer.__callback(records, observer); }
                    catch (error) { setTimeout(() => { throw error; }, 0); }
                }
            }
        });
    }
}
function __mutationBefore(operation, args) {
    if (__mutationObservers.size === 0 && !__callHost('mutationObserved')) return null;
    let target = args[0];
    if (operation.startsWith('style')) target = __styleOwners.get(__styleDeclarationTarget(target));
    if (!__callHost('isNativeNode', target)) return null;
    if (operation.includes('Attribute') || operation.startsWith('style')) {
        const namespaced = operation.endsWith('NS');
        const namespace = namespaced ? args[1] || null : null;
        const name = operation.startsWith('style') ? 'style' : namespaced ? String(args[2]).split(':').at(-1) : String(args[1]);
        const value = namespaced
            ? __attributeRecords(target).find(attribute => attribute.namespaceURI === namespace && attribute.localName === name)?.value ?? null
            : __callHost('getAttribute', target, name);
        return { type: 'attributes', target, name, namespace, value, operation };
    }
    if (operation === 'setTextContent' && [3,4,7,8].includes(__callHost('nodeType', target))) {
        return { type: 'characterData', target, value: __callHost('textContent', target) };
    }
    if (operation === 'setInnerHTML' && target instanceof HTMLTemplateElement) target = target.content;
    if (operation === 'replaceChild') {
        const [parent,node,child] = args, added = node.nodeType === 11 ? Array.from(node.childNodes) : [node];
        const records = [];
        if(node.parentNode || node.nodeType === 11 && added.length) {
            const record = new MutationRecord('childList',node.nodeType === 11 ? node : node.parentNode);
            record.removedNodes = __createNodeList(added);
            if(node.nodeType!==11) { record.previousSibling=node.previousSibling;record.nextSibling=node.nextSibling; }
            records.push(record);
        }
        const record = new MutationRecord('childList',parent);
        record.addedNodes = __createNodeList(added); record.removedNodes = __createNodeList(node===child ? [] : [child]);
        record.previousSibling = child.previousSibling===node ? node.previousSibling : child.previousSibling;
        record.nextSibling = child.nextSibling===node ? node.nextSibling : child.nextSibling;
        records.push(record);
        // A replacement can detach the ancestor of an earlier removal target.
        // Preserve that target's observer ancestry while it is still in the tree.
        return {type:'replacement',records:records.map(record => [record, __mutationAncestors(record.target)])};
    }
    const parents = new Set();
    if (['appendChild', 'insertBefore', 'removeChild'].includes(operation)) {
        const child = args[1];
        const parent = child instanceof Node ? __callHost('parentNode', child) : null;
        if (parent) parents.add(parent);
        if (child?.nodeType === 11) parents.add(child);
    }
    parents.add(target);
    const moved = ['appendChild', 'insertBefore'].includes(operation) && args[1]?.parentNode === target
        ? args[1] : null;
    return { type: 'childList', moved, parents: [...parents].map(parent => [parent, Array.from(parent.childNodes)]) };
}
function __mutationAfter(before) {
    if (!before) return;
    if (before.type === 'replacement') { for(const [record, ancestors] of before.records)__queueMutation(record, ancestors);return; }
    if (before.type !== 'childList') {
        if (before.type === 'attributes' && before.operation.startsWith('removeAttribute') && before.value === null) return;
        if (before.type === 'attributes' && before.operation.startsWith('style') &&
            before.target.getAttribute(before.name) === before.value) return;
        const record = new MutationRecord(before.type, before.target);
        record.oldValue = before.value;
        if (before.type === 'attributes') { record.attributeName = before.name; record.attributeNamespace = before.namespace; }
        __queueMutation(record);
        return;
    }
    for (const [target, previous] of before.parents) {
        const current = Array.from(target.childNodes);
        const previousSet = new Set(previous), currentSet = new Set(current);
        const removed = previous.filter(node => !currentSet.has(node));
        const added = current.filter(node => !previousSet.has(node));
        if (before.moved && previousSet.has(before.moved) && currentSet.has(before.moved)) {
            const removal = new MutationRecord('childList', target);
            const index = previous.indexOf(before.moved);
            removal.removedNodes = __createNodeList([before.moved]);
            removal.previousSibling = previous[index - 1] ?? null;
            removal.nextSibling = previous[index + 1] ?? null;
            __queueMutation(removal);
            added.push(before.moved);
        }
        if (!removed.length && !added.length) continue;
        const record = new MutationRecord('childList', target);
        record.addedNodes = __createNodeList(added); record.removedNodes = __createNodeList(removed);
        const list = added.length ? current : previous;
        const affected = added.length ? added : removed;
        const first = list.indexOf(affected[0]), last = list.indexOf(affected[affected.length - 1]);
        record.previousSibling = list[first - 1] ?? null;
        record.nextSibling = list[last + 1] ?? null;
        __queueMutation(record);
    }
}

const __deliverParserMutations = serialized => {
    __domVersion++;
    __updateStyleSheets();
    if (__liveRanges.size === 0 && __mutationObservers.size === 0 && !__callHost("mutationObserved")) return;
    const node = id => id == null ? null : __callHost('mutationNode', globalThis, String(id));
    for (const item of JSON.parse(serialized)) {
        const record = new MutationRecord(item.oldValue == null ? 'childList' : 'characterData', node(item.target));
        record.oldValue = item.oldValue;
        if (item.oldValue == null) record[item.removed ? 'removedNodes' : 'addedNodes'] = __createNodeList([node(item.node)]);
        record.previousSibling = node(item.previous); record.nextSibling = node(item.next);
        __queueMutation(record);
    }
};

globalThis.__brimpParserMutations = __deliverParserMutations;
