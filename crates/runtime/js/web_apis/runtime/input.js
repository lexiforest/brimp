globalThis.__brimpInputController = (() => {
    const apply = Reflect.apply;
    const dispatchEvent = EventTarget.prototype.dispatchEvent;
    let mouseTarget = null;
    let mouseDownTarget = null;
    let mouseButtons = 0;
    let drag = null;


    function trusted(event) {
        return __markTrustedEvent(event);
    }

    function modifierOptions(modifiers = 0) {
        return {
            altKey: Boolean(modifiers & 1),
            ctrlKey: Boolean(modifiers & 2),
            metaKey: Boolean(modifiers & 4),
            shiftKey: Boolean(modifiers & 8),
        };
    }

    function dispatch(target, event) {
        return apply(dispatchEvent, target, [trusted(event)]);
    }

    function mouseOptions(command, overrides = {}) {
        return {
            bubbles: true,
            cancelable: true,
            composed: true,
            view: window,
            button: command.button,
            buttons: command.buttons,
            detail: command.clickCount,
            ...modifierOptions(command.modifiers),
            ...overrides,
        };
    }

    function pointer(target, type, command, overrides = {}) {
        return dispatch(target, new PointerEvent(type, mouseOptions(command, {
            pointerId: overrides.pointerId ?? 1,
            pointerType: overrides.pointerType ?? command.pointerType ?? "mouse",
            isPrimary: true,
            pressure: command.force ?? (command.buttons ? 0.5 : 0),
            tangentialPressure: command.tangentialPressure ?? 0,
            tiltX: command.tiltX ?? 0, tiltY: command.tiltY ?? 0, twist: command.twist ?? 0,
            ...overrides,
        })));
    }

    function mouse(target, type, command, overrides = {}) {
        return dispatch(target, new MouseEvent(type, mouseOptions(command, overrides)));
    }

    function enterTarget(target, command) {
        if (target === mouseTarget) return;
        if (mouseTarget) {
            pointer(mouseTarget, "pointerout", command, { relatedTarget: target });
            mouse(mouseTarget, "mouseout", command, { relatedTarget: target });
            pointer(mouseTarget, "pointerleave", command, { bubbles: false, relatedTarget: target });
            mouse(mouseTarget, "mouseleave", command, { bubbles: false, relatedTarget: target });
        }
        if (target) {
            pointer(target, "pointerover", command, { relatedTarget: mouseTarget });
            mouse(target, "mouseover", command, { relatedTarget: mouseTarget });
            pointer(target, "pointerenter", command, { bubbles: false, relatedTarget: mouseTarget });
            mouse(target, "mouseenter", command, { bubbles: false, relatedTarget: mouseTarget });
        }
        mouseTarget = target;
    }

    function focus(target) {
        if (!target || target.disabled || document.activeElement === target) return;
        const previous = document.activeElement;
        if (previous) {
            dispatch(previous, new FocusEvent("blur", {relatedTarget: target}));
            dispatch(previous, new FocusEvent("focusout", { bubbles: true, composed: true, relatedTarget: target }));
        }
        __activeElement = target;
        dispatch(target, new FocusEvent("focus", {relatedTarget: previous}));
        dispatch(target, new FocusEvent("focusin", { bubbles: true, composed: true, relatedTarget: previous }));
    }

    function focusableTarget(target) {
        return target?.closest?.("input,textarea,select,button,a[href],[tabindex],[contenteditable]") ?? null;
    }

    function inputEvent(target, type = "input") {
        dispatch(target, new Event(type, { bubbles: true, composed: true }));
    }

    function activationElement(target) {
        return target?.closest?.("button,input,label,a[href]") ?? target;
    }

    function prepareActivation(target) {
        target = activationElement(target);
        if (!(target instanceof HTMLInputElement) || target.disabled) return null;
        if (target.type === "checkbox") {
            const checked = Boolean(target.checked);
            __inputCheckedStates.set(target, !checked);
            return accepted => {
                if (!accepted) __inputCheckedStates.set(target, checked);
                else {
                    inputEvent(target);
                    inputEvent(target, "change");
                }
            };
        }
        if (target.type === "radio" && !target.checked) {
            const radios = target.name
                ? Array.from(document.querySelectorAll('input[type="radio"]')).filter(radio => radio.name === target.name)
                : [target];
            const checked = radios.map(radio => Boolean(radio.checked));
            for (const radio of radios) __inputCheckedStates.set(radio, radio === target);
            return accepted => {
                if (!accepted) radios.forEach((radio, index) => { __inputCheckedStates.set(radio, checked[index]); });
                else {
                    inputEvent(target);
                    inputEvent(target, "change");
                }
            };
        }
        return null;
    }

    function activate(target) {
        target = activationElement(target);
        if (!target || target.disabled) return;
        if (target instanceof HTMLLabelElement) {
            const control = target.htmlFor
                ? document.getElementById(target.htmlFor)
                : target.querySelector("input,button,select,textarea");
            if (control) clickElement(control, 1, 0);
            return;
        }
        if (target instanceof HTMLInputElement) {
            if (target.type === "submit") {
                submit(__formOwner(target));
            } else if (target.type === "reset") {
                reset(__formOwner(target));
            }
        } else if (target instanceof HTMLButtonElement) {
            if (target.type === "submit") submit(__formOwner(target));
            else if (target.type === "reset") reset(__formOwner(target));
        } else if (target instanceof HTMLAnchorElement && target.href) {
            location.href = target.href;
        }
    }

    function submit(form) { if (form) form.requestSubmit(); }

    function reset(form) { if (form) form.reset(); }

    function releaseClick(target, command) {
        const activationTarget = activationElement(target);
        if (mouseDownTarget !== target || activationTarget?.disabled) return;
        if (command.button === 0) {
            const prepared = prepareActivation(target);
            const accepted = pointer(target, "click", command);
            prepared?.(accepted);
            if (accepted && !prepared) activate(target);
            if (command.clickCount === 2) mouse(target, "dblclick", command);
        } else if (command.button === 1) {
            mouse(target, "auxclick", command);
        } else if (command.button === 2) {
            mouse(target, "contextmenu", command);
        }
    }

    function moveElement(target, clickCount = 0, modifiers = 0) {
        const command = {button: 0, buttons: 0, clickCount, modifiers};
        enterTarget(target, command);
        pointer(target, 'pointermove', command);
        mouse(target, 'mousemove', command);
        return command;
    }
    function clickElement(target, clickCount = 1, modifiers = 0) {
        const command = moveElement(target, clickCount, modifiers);
        __notifyUserActivation();
        mouseDownTarget = target;
        command.buttons = mouseButtons = 1;
        const accepted = pointer(target, 'pointerdown', command);
        if (accepted && mouse(target, 'mousedown', command)) focus(focusableTarget(target));
        command.buttons = mouseButtons = 0;
        pointer(target, 'pointerup', command);
        if (accepted) mouse(target, 'mouseup', command);
        releaseClick(target, command);
        mouseDownTarget = null;
    }

    const dragEffects = new Map([[0,"none"],[1,"copy"],[2,"link"],[3,"copyLink"],[16,"move"],[17,"copyMove"],[18,"linkMove"],[19,"all"]]);
    function dragEvent(target, type, command, data) {
        if (!target) return true;
        return dispatch(target, new DragEvent(type, mouseOptions(command, {dataTransfer:data, cancelable:type !== "dragleave" && type !== "dragend"})));
    }
    function startDrag(source, command) {
        if (!source || source.getAttribute("draggable") === "false") return;
        const data = new DataTransfer();
        data.effectAllowed = "all";
        if (source.href || source.src) { data.setData("text/uri-list", source.href || source.src); data.setData("text/plain", source.textContent || source.href || source.src); }
        if (!dragEvent(source, "dragstart", command, data)) { mouseDownTarget = null; return; }
        data.__readOnly = true;
        drag = {source, target:null, data, accepted:false, };
        pointer(source, "pointercancel", command);
    }
    function updateDrag(target, command) {
        if (target !== drag.target) {
            dragEvent(drag.target, "dragleave", command, drag.data);
            dragEvent(target, "dragenter", command, drag.data);
            drag.target = target;
        }
        const effect = drag.data.effectAllowed;
        drag.data.dropEffect = effect.includes("copy") || effect === "all" || effect === "uninitialized" ? "copy" : effect.includes("move") ? "move" : effect.includes("link") ? "link" : "none";
        drag.accepted = !dragEvent(target, "dragover", command, drag.data);
        if (target instanceof HTMLInputElement && target.type === "file") drag.accepted = true;
    }
    function finishDrag(target, command, cancelled) {
        if (!drag) return;
        const mask = Array.from(dragEffects).find(([,effect]) => effect === drag.data.effectAllowed)?.[0] ?? 19;
        const operation = {copy:1,link:2,move:16}[drag.data.dropEffect] || 0;
        cancelled ||= !(mask & operation);
        if (!cancelled && drag.accepted && target && drag.data.dropEffect !== "none") {
            const accepted = dragEvent(target, "drop", command, drag.data);
            if (accepted && target instanceof HTMLInputElement && target.type === "file") {
                const files = Array.from(drag.data.files);
                target.__setFiles(target.multiple ? files : files.slice(0,1));
                inputEvent(target); inputEvent(target,"change");
            }
        } else { drag.data.dropEffect = "none"; dragEvent(drag.target, "dragleave", command, drag.data); }
        dragEvent(drag.source, "dragend", command, drag.data);
        drag = null; mouseDownTarget = null; mouseButtons = 0;
    }
    function dragTo(source, target) {
        const command = moveElement(source);
        startDrag(source, command);
        if (drag) { updateDrag(target, command); finishDrag(target, command, false); }
    }

    function editableTarget() {
        const target = document.activeElement;
        if (!target || target.disabled || target.readOnly) return null;
        if (target instanceof HTMLTextAreaElement) return target;
        if (target instanceof HTMLInputElement && ["text","search","url","tel","email","password","number"].includes(target.type)) return target;
        return target.closest?.('[contenteditable="true"],[contenteditable=""]') || null;
    }
    function selection(target) {
        if ("value" in target) return {value:String(target.value),start:target.selectionStart,end:target.selectionEnd};
        const value = target.textContent;
        const range = document.getSelection()?.rangeCount ? document.getSelection().getRangeAt(0) : null;
        if (range && range.startContainer === target.firstChild && range.endContainer === target.firstChild) return {value,start:range.startOffset,end:range.endOffset};
        return {value,start:value.length,end:value.length};
    }
    function caret(target,start,end=start) {
        if ("value" in target) target.setSelectionRange(start,end);
        else if (target.firstChild) {
            const range = document.createRange(); range.setStart(target.firstChild,start); range.setEnd(target.firstChild,end);
            const selected = document.getSelection(); selected.removeAllRanges(); selected.addRange(range);
        }
    }
    function replaceText(target,text,inputType="insertText",start,end) {
        const selected = selection(target);
        start ??= selected.start; end ??= selected.end;
        if (!dispatch(target,new InputEvent("beforeinput",{bubbles:true,cancelable:true,composed:true,data:text || null,inputType}))) return false;
        const value = selected.value.slice(0,start) + text + selected.value.slice(end);
        if (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement) __inputValues.set(target, value);
        else if ("value" in target) target.value = value; else target.textContent = value;
        caret(target,start+text.length);
        dispatch(target,new InputEvent("input",{bubbles:true,composed:true,data:text || null,inputType}));
        return true;
    }
    function edit(target, key, text, modifiers) {
        if (!target) return;
        const {value,start,end} = selection(target);
        if ((modifiers & 6) && key.toLowerCase() === "a") { caret(target,0,value.length); return; }
        if (modifiers & 6) return;
        if (key === "ArrowLeft") { caret(target,Math.max(0,start-(start===end ? (Array.from(value.slice(0,start)).pop()?.length || 1) : 0))); return; }
        if (key === "ArrowRight") { caret(target,Math.min(value.length,end+(start===end ? (Array.from(value.slice(end))[0]?.length || 1) : 0))); return; }
        if (key === "Home") { caret(target,0); return; }
        if (key === "End") { caret(target,value.length); return; }
        if (key === "Backspace") replaceText(target,"","deleteContentBackward",start===end ? Math.max(0,start-(Array.from(value.slice(0,start)).pop()?.length || 1)) : start,end);
        else if (key === "Delete") replaceText(target,"","deleteContentForward",start,start===end ? Math.min(value.length,end+(Array.from(value.slice(end))[0]?.length || 1)) : end);
        else if (key === "Enter" && !(target instanceof HTMLInputElement)) replaceText(target,"\n","insertLineBreak");
        else if (text) replaceText(target,text);
    }

    function tab(command) {
        const controls = Array.from(document.querySelectorAll("input,textarea,select,button,a[href],[tabindex],[contenteditable]"))
            .filter(element => !element.disabled && element.getAttribute("tabindex") !== "-1");
        if (!controls.length) return;
        let index = controls.indexOf(document.activeElement);
        index += command.modifiers & 8 ? -1 : 1;
        if (index < 0) index = controls.length - 1;
        focus(controls[index % controls.length]);
    }

    function dispatchKey(command) {
        const target = document.activeElement || document.body;
        if (!target) return false;
        command.modifiers = Number(command.modifiers ?? 0);
        const type = ({ keyDown: "keydown", rawKeyDown: "keydown", keyUp: "keyup", char: "keypress" })[command.eventType];
        if (!type) throw new TypeError(`unsupported key event type: ${command.eventType}`);
        if ((command.eventType === "keyDown" || command.eventType === "rawKeyDown") && command.key !== "Escape") {
            __notifyUserActivation();
        }
        const event = new KeyboardEvent(type, {
            bubbles: true,
            cancelable: true,
            composed: true,
            key: command.key ?? "",
            code: command.code ?? "",
            repeat: Boolean(command.autoRepeat),
            location: command.isKeypad ? 3 : (command.location ?? 0),
            keyCode: command.windowsVirtualKeyCode ?? command.nativeVirtualKeyCode ?? 0,
            charCode: command.eventType === "char" ? (command.text || "").codePointAt(0) || 0 : 0,
            ...modifierOptions(command.modifiers),
        });
        const accepted = dispatch(target, event);
        if (accepted && command.key === "Escape") finishDrag(null, {}, true);
        if (accepted && ["keyDown", "rawKeyDown", "char"].includes(command.eventType)) {
            if (command.key === "Tab") tab(command);
            else if (command.key === "Enter" && target instanceof HTMLInputElement) submit(__formOwner(target));
            else if (command.commands?.length) {
                const keys = {selectAll:"a",deleteBackward:"Backspace",deleteForward:"Delete",insertNewline:"Enter",moveLeft:"ArrowLeft",moveRight:"ArrowRight",moveToBeginningOfLine:"Home",moveToEndOfLine:"End"};
                for (const name of command.commands) edit(editableTarget(),keys[name],"",name === "selectAll" ? 2 : 0);
            } else {
                const text = command.eventType === "rawKeyDown" ? "" : command.text ?? "";
                let characterAccepted = true;
                if (text && command.eventType === "keyDown") characterAccepted = dispatch(target,new KeyboardEvent("keypress",{bubbles:true,cancelable:true,composed:true,key:command.key,code:command.code,charCode:text.codePointAt(0),...modifierOptions(command.modifiers)}));
                if (characterAccepted) edit(editableTarget(),command.key ?? "",text,command.modifiers);
            }
        }
        return true;
    }

    function insertText(text) {
        const target = editableTarget();
        if (!target) throw new Error("BRIMP_INPUT_NO_FOCUS");
        replaceText(target,String(text));
        return true;
    }

    function tapElement(target, modifiers = 0) {
        const command = {button: 0, buttons: 1, clickCount: 1, modifiers, pointerType: 'touch'};
        const point = new Touch({identifier: 0, target, force: 1});
        pointer(target, 'pointerover', command);
        pointer(target, 'pointerenter', command, {bubbles: false});
        pointer(target, 'pointerdown', command);
        const accepted = dispatch(target, new TouchEvent('touchstart', {bubbles: true, cancelable: true, composed: true, touches: [point], targetTouches: [point], changedTouches: [point], ...modifierOptions(modifiers)}));
        __notifyUserActivation();
        command.buttons = 0;
        pointer(target, 'pointerup', command);
        const ended = dispatch(target, new TouchEvent('touchend', {bubbles: true, cancelable: true, composed: true, touches: [], targetTouches: [], changedTouches: [point], ...modifierOptions(modifiers)}));
        pointer(target, 'pointerout', command);
        pointer(target, 'pointerleave', command, {bubbles: false});
        if (accepted && ended && !target.disabled) {
            mouse(target, 'mouseover', command); mouse(target, 'mousemove', command);
            command.buttons = 1;
            if (mouse(target, 'mousedown', command)) focus(focusableTarget(target));
            command.buttons = 0; mouse(target, 'mouseup', command);
            mouseDownTarget = target; releaseClick(target, command); mouseDownTarget = null;
        }
    }

    function targetForSelector(selector) {
        const target = document.querySelector(String(selector));
        if (!target) throw new Error(`BRIMP_INPUT_NOT_FOUND:${selector}`);
        return target;
    }

    function typeText(target, text) {
        focus(target);
        for (const character of Array.from(String(text))) {
            const command = { eventType: "keyDown", key: character, code: "", text: character, modifiers: 0 };
            dispatchKey(command);
            dispatchKey({ ...command, eventType: "keyUp", text: "" });
        }
    }

    function fill(target, text) {
        focus(target);
        if (editableTarget() !== target) throw new Error('element is not editable');
        if (replaceText(target, String(text), 'insertText', 0, selection(target).value.length)) inputEvent(target, 'change');
    }
    function files(target, uploads) {
        if (!(target instanceof HTMLInputElement) || target.type !== 'file') throw new TypeError('target must be a file input');
        if (!target.multiple && uploads.length > 1) throw new TypeError('input does not accept multiple files');
        target.__setFiles(uploads.map(file => new File([Uint8Array.from(file.bytes)], file.name, {type: file.mimeType ?? '', lastModified: file.lastModified ?? 0})));
        inputEvent(target); inputEvent(target, 'change');
    }
    return (serialized, targetOverride) => {
        const command = JSON.parse(String(serialized));
        const target = targetOverride ?? (command.selector == null ? document.activeElement : targetForSelector(command.selector));
        if (!target || !(target instanceof Element)) throw new TypeError('input requires an element');
        if (command.action === 'click') clickElement(target, command.clickCount ?? 1, command.modifiers ?? 0);
        else if (command.action === 'hover') moveElement(target, 0, command.modifiers ?? 0);
        else if (command.action === 'type') typeText(target, command.text);
        else if (command.action === 'tap') tapElement(target, command.modifiers ?? 0);
        else if (command.action === 'focus') focus(target);
        else if (command.action === 'fill') fill(target, command.text);
        else if (command.action === 'files') files(target, command.files);
        else if (command.action === 'dragTo') dragTo(target, targetForSelector(command.destination));
        else if (command.action === 'key') { focus(target); dispatchKey(command); }
        else if (command.action === 'press') {
            focus(target);
            const down = {...command, eventType: 'keyDown', text: Array.from(command.key).length === 1 ? command.key : ''};
            dispatchKey(down); dispatchKey({...down, eventType: 'keyUp', text: ''});
        }
        else throw new TypeError(`unsupported input action: ${command.action}`);
        return 'true';
    };
})();
