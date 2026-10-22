class MutationRecord {
    constructor(type, target) {
        this.type = type;
        this.target = target;
        this.addedNodes = new NodeList([]);
        this.removedNodes = new NodeList([]);
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
function __queueMutation(record) {
    if (__callHost('mutationObserved')) {
        const id = node => __callHost('mutationNodeId', node);
        __callHost('mutationEvent', globalThis, JSON.stringify({kind: record.type, target: id(record.target), added: Array.from(record.addedNodes, id), removed: Array.from(record.removedNodes, id), attribute: record.attributeName, oldValue: record.oldValue}));
    }
    for (const observer of __mutationObservers) {
        let interested = false, oldValue = false;
        for (const [root, options] of [...observer.__targets, ...observer.__transient]) {
            if (root !== record.target && !(options.subtree && root.contains(record.target))) continue;
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
    if (!(target instanceof Node)) return null;
    if (operation === 'setAttribute' || operation === 'removeAttribute' || operation.startsWith('style')) {
        const name = operation.startsWith('style') ? 'style' : String(args[1]);
        return { type: 'attributes', target, name, value: target.getAttribute(name), operation };
    }
    if (operation === 'setTextContent' && (target.nodeType === 3 || target.nodeType === 8)) {
        return { type: 'characterData', target, value: target.textContent };
    }
    if (operation === 'setInnerHTML' && target instanceof HTMLTemplateElement) target = target.content;
    const parents = new Set();
    if (['appendChild', 'insertBefore', 'removeChild'].includes(operation)) {
        const child = args[1];
        if (child?.parentNode) parents.add(child.parentNode);
        if (child?.nodeType === 11) parents.add(child);
    }
    parents.add(target);
    const moved = ['appendChild', 'insertBefore'].includes(operation) && args[1]?.parentNode === target
        ? args[1] : null;
    return { type: 'childList', moved, parents: [...parents].map(parent => [parent, Array.from(parent.childNodes)]) };
}
function __mutationAfter(before) {
    if (!before) return;
    if (before.type !== 'childList') {
        if (before.type === 'attributes' && before.operation === 'removeAttribute' && before.value === null) return;
        if (before.type === 'attributes' && before.operation.startsWith('style') &&
            before.target.getAttribute(before.name) === before.value) return;
        const record = new MutationRecord(before.type, before.target);
        record.oldValue = before.value;
        if (before.type === 'attributes') record.attributeName = before.name;
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
            removal.removedNodes = new NodeList([before.moved]);
            removal.previousSibling = previous[index - 1] ?? null;
            removal.nextSibling = previous[index + 1] ?? null;
            __queueMutation(removal);
            added.push(before.moved);
        }
        if (!removed.length && !added.length) continue;
        const record = new MutationRecord('childList', target);
        record.addedNodes = new NodeList(added); record.removedNodes = new NodeList(removed);
        const list = added.length ? current : previous;
        const affected = added.length ? added : removed;
        const first = list.indexOf(affected[0]), last = list.indexOf(affected[affected.length - 1]);
        record.previousSibling = list[first - 1] ?? null;
        record.nextSibling = list[last + 1] ?? null;
        __queueMutation(record);
    }
}

globalThis.__brimpParserMutations = serialized => {
    if (__liveRanges.size === 0 && __mutationObservers.size === 0 && !__callHost("mutationObserved")) return;
    const node = id => id == null ? null : __callHost('mutationNode', globalThis, String(id));
    for (const item of JSON.parse(serialized)) {
        const record = new MutationRecord(item.oldValue == null ? 'childList' : 'characterData', node(item.target));
        record.oldValue = item.oldValue;
        if (item.oldValue == null) record[item.removed ? 'removedNodes' : 'addedNodes'] = new NodeList([node(item.node)]);
        record.previousSibling = node(item.previous); record.nextSibling = node(item.next);
        __queueMutation(record);
    }
};
