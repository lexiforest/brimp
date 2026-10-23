// Iterators hold weak registrations so discarded iterators do not retain trees.
const __nodeIterators = new Set();
const __nodeIteratorToken = {};
function __followingNode(node, root, skipChildren = false) {
    if (!skipChildren && node.firstChild) return node.firstChild;
    while (node !== root) {
        if (node.nextSibling) return node.nextSibling;
        node = node.parentNode;
        if (!node) return null;
    }
    return null;
}
function __precedingNode(node, root) {
    if (node === root) return null;
    if (!node.previousSibling) return node.parentNode;
    node = node.previousSibling;
    while (node.lastChild) node = node.lastChild;
    return node;
}
class NodeIterator {
    #root; #show; #filter; #reference; #before = true; #active = false;
    constructor(root, whatToShow, filter, token) {
        if (token !== __nodeIteratorToken) throw new TypeError("Illegal constructor");
        this.#root = this.#reference = root;
        this.#show = Number(whatToShow) >>> 0;
        this.#filter = filter;
        __nodeIterators.add(new WeakRef(this));
    }
    get root() { return this.#root; }
    get whatToShow() { return this.#show; }
    get filter() { return this.#filter; }
    get referenceNode() { return this.#reference; }
    get pointerBeforeReferenceNode() { return this.#before; }
    #accept(node) {
        if (this.#active) throw new DOMException('Recursive node filtering', 'InvalidStateError');
        if (!(this.#show & (1 << (node.nodeType - 1)))) return false;
        if (this.#filter == null) return true;
        this.#active = true;
        try {
            return Number(typeof this.#filter === 'function' ? this.#filter(node) : this.#filter.acceptNode(node)) === NodeFilter.FILTER_ACCEPT;
        } finally { this.#active = false; }
    }
    #traverse(forward) {
        let node = this.#reference, before = this.#before;
        while (true) {
            if (forward) { if (!before) node = __followingNode(node, this.#root); before = false; }
            else { if (before) node = __precedingNode(node, this.#root); before = true; }
            if (!node) return null;
            if (this.#accept(node)) { this.#reference = node; this.#before = before; return node; }
        }
    }
    nextNode() { return this.#traverse(true); }
    previousNode() { return this.#traverse(false); }
    detach() {} // Per DOM, detach no longer changes traversal or registration.
    __beforeRemoval(removedNodes) {
        const removed = removedNodes.find(node => node !== this.#root && node.contains(this.#reference) && this.#root.contains(node));
        if (!removed) return null;
        let next = this.#before ? __followingNode(removed, this.#root, true) : null;
        while (next && removedNodes.some(node => node.contains(next))) next = __followingNode(next, this.#root, true);
        const before = !!next;
        if (!next) {
            next = __precedingNode(removed, this.#root);
            while (next && removedNodes.some(node => node.contains(next))) next = __precedingNode(next, this.#root);
        }
        return () => { this.#reference = next; this.#before = before; };
    }
}
function __iteratorMutationBefore(operation, args) {
    if (!__nodeIterators.size) return [];
    let removed = [];
    const [target, child] = args;
    if (operation === 'removeChild' && child?.parentNode === target) removed = [child];
    if (operation === 'replaceChild') removed = [args[2], ...(child?.nodeType === 11 ? child.childNodes : child?.parentNode ? [child] : [])];
    if (operation === 'appendChild' || operation === 'insertBefore') {
        if (child?.nodeType === 11) removed = [...child.childNodes];
        else if (child?.parentNode) removed = [child];
    }
    if (operation === 'setInnerHTML' || (operation === 'setTextContent' && target?.nodeType !== 3 && target?.nodeType !== 8)) {
        removed = [...(target instanceof HTMLTemplateElement ? target.content : target).childNodes];
    }
    const updates = [];
    for (const ref of __nodeIterators) {
        const iterator = ref.deref();
        if (!iterator) { __nodeIterators.delete(ref); continue; }
        const update = iterator.__beforeRemoval(removed);
        if (update) updates.push(update);
    }
    return updates;
}
