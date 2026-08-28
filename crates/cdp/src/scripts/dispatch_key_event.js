(() => {
    const target = document.activeElement || document.body;
    const event = new KeyboardEvent(__DOM_EVENT__, {
        bubbles: true,
        cancelable: true,
        key: __KEY__,
        code: __CODE__,
        repeat: __REPEAT__,
        altKey: Boolean(__MODIFIERS__ & 1),
        ctrlKey: Boolean(__MODIFIERS__ & 2),
        metaKey: Boolean(__MODIFIERS__ & 4),
        shiftKey: Boolean(__MODIFIERS__ & 8),
    });
    const accepted = target.dispatchEvent(event);
    if (
        accepted
        && ["keyDown", "rawKeyDown", "char"].includes(__EVENT_TYPE__)
    ) {
        const insertion = __TEXT__;
        if (__KEY__ === "Backspace") {
            target.value = Array.from(String(target.value || "")).slice(0, -1).join("");
            target.dispatchEvent(new Event("input", { bubbles: true }));
        } else if (insertion && "value" in target) {
            target.value = String(target.value || "") + insertion;
            target.dispatchEvent(new Event("input", { bubbles: true }));
        }
    }
    return true;
})()
