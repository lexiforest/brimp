(() => {
    const target = document.activeElement;
    if (!target || !("value" in target)) return false;
    target.value = String(target.value || "") + __TEXT__;
    target.dispatchEvent(new Event("input", { bubbles: true }));
    return true;
})()
