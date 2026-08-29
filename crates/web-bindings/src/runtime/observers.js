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

class MutationObserver {
    constructor(callback) {
        if (typeof callback !== "function") throw new TypeError("MutationObserver callback must be a function");
        this.__callback = callback;
        this.__target = null;
        this.__timer = null;
        this.__snapshot = "";
    }
    observe(target, options = {}) {
        if (!(target instanceof Node)) throw new TypeError("MutationObserver target must be a Node");
        if (!options.childList && !options.attributes && !options.characterData) {
            throw new TypeError("MutationObserver options must enable childList, attributes, or characterData");
        }
        this.disconnect();
        this.__target = target;
        this.__snapshot = this.__takeSnapshot();
        this.__schedule();
    }
    disconnect() {
        if (this.__timer !== null) clearTimeout(this.__timer);
        this.__timer = null;
        this.__target = null;
    }
    takeRecords() { return []; }
    __takeSnapshot() {
        if (!this.__target) return "";
        if (this.__target.nodeType === 9) {
            return this.__target.documentElement ? this.__target.documentElement.outerHTML : "";
        }
        return this.__target.outerHTML === undefined
            ? String(this.__target.textContent || "")
            : String(this.__target.outerHTML);
    }
    __schedule() {
        this.__timer = setTimeout(() => {
            if (!this.__target) return;
            const snapshot = this.__takeSnapshot();
            if (snapshot !== this.__snapshot) {
                this.__snapshot = snapshot;
                this.__callback([new MutationRecord("childList", this.__target)], this);
            }
            if (this.__target) this.__schedule();
        }, 16);
    }
}

class IntersectionObserverEntry {
    constructor(target, rootBounds, boundingClientRect, intersectionRect) {
        this.time = performance.now();
        this.target = target;
        this.rootBounds = rootBounds;
        this.boundingClientRect = boundingClientRect;
        this.intersectionRect = intersectionRect;
        const targetArea = boundingClientRect.width * boundingClientRect.height;
        const intersectionArea = intersectionRect.width * intersectionRect.height;
        this.intersectionRatio = targetArea === 0 ? 0 : intersectionArea / targetArea;
        this.isIntersecting = intersectionArea > 0;
    }
}

class IntersectionObserver {
    constructor(callback, options = {}) {
        if (typeof callback !== "function") throw new TypeError("IntersectionObserver callback must be a function");
        this.__callback = callback;
        this.root = options.root || null;
        this.rootMargin = String(options.rootMargin || "0px");
        const threshold = options.threshold === undefined ? [0] :
            (Array.isArray(options.threshold) ? options.threshold : [options.threshold]);
        this.thresholds = threshold.map(Number).sort((left, right) => left - right);
        if (this.thresholds.some(value => !Number.isFinite(value) || value < 0 || value > 1)) {
            throw new RangeError("IntersectionObserver threshold must be between 0 and 1");
        }
        this.__targets = new Set();
        this.__timer = null;
    }
    observe(target) {
        if (!(target instanceof Element)) throw new TypeError("IntersectionObserver target must be an Element");
        this.__targets.add(target);
        if (this.__timer === null) {
            this.__timer = setTimeout(() => {
                this.__timer = null;
                const rootBounds = this.root ? this.root.getBoundingClientRect() :
                    new DOMRect(0, 0, window.innerWidth, window.innerHeight);
                const entries = [...this.__targets].map(target => {
                    const rect = target.getBoundingClientRect();
                    const left = Math.max(rect.left, rootBounds.left);
                    const top = Math.max(rect.top, rootBounds.top);
                    const right = Math.min(rect.right, rootBounds.right);
                    const bottom = Math.min(rect.bottom, rootBounds.bottom);
                    const intersection = right > left && bottom > top
                        ? new DOMRect(left, top, right - left, bottom - top)
                        : new DOMRect();
                    return new IntersectionObserverEntry(target, rootBounds, rect, intersection);
                });
                if (entries.length) this.__callback(entries, this);
            }, 0);
        }
    }
    unobserve(target) { this.__targets.delete(target); }
    disconnect() {
        if (this.__timer !== null) clearTimeout(this.__timer);
        this.__timer = null;
        this.__targets.clear();
    }
    takeRecords() { return []; }
}

