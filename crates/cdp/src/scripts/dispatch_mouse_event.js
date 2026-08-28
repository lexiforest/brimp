(() => {
    const target = document.elementFromPoint(__X__, __Y__);
    if (!target) return true;
    const options = {
        bubbles: true,
        cancelable: true,
        view: window,
        clientX: __X__,
        clientY: __Y__,
        button: __BUTTON__,
        buttons: __BUTTONS__,
        detail: __CLICK_COUNT__,
        altKey: Boolean(__MODIFIERS__ & 1),
        ctrlKey: Boolean(__MODIFIERS__ & 2),
        metaKey: Boolean(__MODIFIERS__ & 4),
        shiftKey: Boolean(__MODIFIERS__ & 8),
    };
    target.dispatchEvent(new MouseEvent(__DOM_EVENT__, options));
    if (__EVENT_TYPE__ === "mousePressed") globalThis.__brimpCdpMouseDownTarget = target;
    if (__EVENT_TYPE__ === "mouseReleased") {
        if (globalThis.__brimpCdpMouseDownTarget === target && __BUTTON__ === 0) {
            if (typeof target.focus === "function") target.focus();
            target.dispatchEvent(new MouseEvent("click", options));
            if (__CLICK_COUNT__ === 2) target.dispatchEvent(new MouseEvent("dblclick", options));
        }
        delete globalThis.__brimpCdpMouseDownTarget;
    }
    return true;
})()
