globalThis.__brimpInputController = (() => {
    const apply = Reflect.apply;
    const dispatchEvent = EventTarget.prototype.dispatchEvent;
    let mouseTarget = null;
    let mouseDownTarget = null;
    let mouseX = 0;
    let mouseY = 0;
    let mouseButtons = 0;
    const activeTouches = new Map();

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
            clientX: command.x,
            clientY: command.y,
            screenX: command.x,
            screenY: command.y,
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
            pointerType: overrides.pointerType ?? "mouse",
            isPrimary: true,
            pressure: command.buttons ? 0.5 : 0,
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
            dispatch(previous, new Event("blur"));
            dispatch(previous, new Event("focusout", { bubbles: true, composed: true }));
        }
        __activeElement = target;
        dispatch(target, new Event("focus"));
        dispatch(target, new Event("focusin", { bubbles: true, composed: true }));
    }

    function focusableTarget(target) {
        return target?.closest?.("input,textarea,select,button,a[href],[tabindex]") ?? null;
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
            target.checked = !checked;
            return accepted => {
                if (!accepted) target.checked = checked;
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
            for (const radio of radios) radio.checked = radio === target;
            return accepted => {
                if (!accepted) radios.forEach((radio, index) => { radio.checked = checked[index]; });
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
                submit(target.closest("form"));
            } else if (target.type === "reset") {
                reset(target.closest("form"));
            }
        } else if (target instanceof HTMLButtonElement) {
            if (target.type === "submit") submit(target.closest("form"));
            else if (target.type === "reset") reset(target.closest("form"));
        } else if (target instanceof HTMLAnchorElement && target.href) {
            location.href = target.href;
        }
    }

    function submit(form) {
        if (!form) return;
        if (dispatch(form, new Event("submit", { bubbles: true, cancelable: true, composed: true }))) {
            form.submit();
        }
    }

    function reset(form) {
        if (!form || !dispatch(form, new Event("reset", { bubbles: true, cancelable: true, composed: true }))) return;
        for (const control of form.querySelectorAll("input,textarea,select")) {
            if (control instanceof HTMLInputElement) {
                control.value = control.defaultValue;
                control.checked = control.defaultChecked;
            } else if (control instanceof HTMLTextAreaElement) {
                control.value = control.textContent;
            }
        }
    }

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

    function dispatchMouse(command) {
        mouseX = Number(command.x);
        mouseY = Number(command.y);
        command.x = mouseX;
        command.y = mouseY;
        command.button = Number(command.button ?? 0);
        command.buttons = Number(command.buttons ?? mouseButtons);
        command.clickCount = Number(command.clickCount ?? 0);
        command.modifiers = Number(command.modifiers ?? 0);
        const target = document.elementFromPoint(mouseX, mouseY);
        enterTarget(target, command);
        if (!target) return false;
        if (command.eventType === "mouseMoved") {
            pointer(target, "pointermove", command);
            mouse(target, "mousemove", command);
        } else if (command.eventType === "mousePressed") {
            __notifyUserActivation();
            mouseButtons = command.buttons;
            pointer(target, "pointerdown", command);
            const accepted = mouse(target, "mousedown", command);
            mouseDownTarget = target;
            if (accepted && command.button === 0) focus(focusableTarget(target));
        } else if (command.eventType === "mouseReleased") {
            pointer(target, "pointerup", command);
            mouse(target, "mouseup", command);
            mouseButtons = command.buttons;
            releaseClick(target, command);
            mouseDownTarget = null;
        } else {
            throw new TypeError(`unsupported mouse event type: ${command.eventType}`);
        }
        return true;
    }

    function editableTarget() {
        const target = document.activeElement;
        if (!(target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement)
            || target.disabled || target.readOnly) return null;
        return target;
    }

    function edit(target, key, text, modifiers) {
        if (!target || (modifiers & 6)) return;
        let value = String(target.value ?? "");
        if (key === "Backspace") value = Array.from(value).slice(0, -1).join("");
        else if (key === "Delete") return;
        else if (key === "Enter" && target instanceof HTMLTextAreaElement) value += "\n";
        else if (text) value += text;
        else return;
        target.value = value;
        inputEvent(target);
    }

    function tab(command) {
        const controls = Array.from(document.querySelectorAll("input,textarea,select,button,a[href],[tabindex]"))
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
            ...modifierOptions(command.modifiers),
        });
        const accepted = dispatch(target, event);
        if (accepted && ["keyDown", "rawKeyDown", "char"].includes(command.eventType)) {
            if (command.key === "Tab") tab(command);
            else if (command.key === "Enter" && target instanceof HTMLInputElement) submit(target.closest("form"));
            else edit(editableTarget(), command.key, command.text ?? "", command.modifiers);
        }
        return true;
    }

    function insertText(text) {
        const target = editableTarget();
        if (!target) throw new Error("BRIMP_INPUT_NO_FOCUS");
        target.value = String(target.value ?? "") + String(text);
        inputEvent(target);
        return true;
    }

    function touchFromPoint(point, target) {
        return new Touch({
            identifier: point.id,
            target,
            clientX: point.x,
            clientY: point.y,
            radiusX: point.radiusX,
            radiusY: point.radiusY,
            rotationAngle: point.rotationAngle,
            force: point.force,
        });
    }

    function touchPointer(target, type, point, modifiers, buttons) {
        pointer(target, type, {
            x: point.x,
            y: point.y,
            button: 0,
            buttons,
            clickCount: 1,
            modifiers,
        }, {
            pointerId: point.id + 2,
            pointerType: "touch",
            width: point.radiusX * 2,
            height: point.radiusY * 2,
            pressure: buttons ? point.force : 0,
            tangentialPressure: point.tangentialPressure,
        });
    }

    function dispatchTouchToTargets(type, changedStates, modifiers) {
        const touches = Array.from(activeTouches.values(), state => state.touch);
        const targets = Array.from(new Set(changedStates.map(state => state.target)));
        let accepted = true;
        for (const target of targets) {
            const targetTouches = touches.filter(touch => touch.target === target);
            const changedTouches = changedStates
                .filter(state => state.target === target)
                .map(state => state.touch);
            accepted = dispatch(target, new TouchEvent(type, {
                bubbles: true,
                cancelable: true,
                composed: true,
                touches,
                targetTouches,
                changedTouches,
                ...modifierOptions(modifiers),
            })) && accepted;
        }
        return accepted;
    }

    function compatibilityClick(state, modifiers) {
        const command = {
            x: state.point.x,
            y: state.point.y,
            button: 0,
            buttons: 0,
            clickCount: 1,
            modifiers,
        };
        mouse(state.target, "mouseover", command);
        mouse(state.target, "mousemove", command);
        command.buttons = 1;
        const accepted = mouse(state.target, "mousedown", command);
        if (accepted) focus(focusableTarget(state.target));
        command.buttons = 0;
        mouse(state.target, "mouseup", command);
        const activationTarget = activationElement(state.target);
        const prepared = prepareActivation(state.target);
        const clickAccepted = !activationTarget?.disabled && pointer(state.target, "click", command, {
            pointerId: state.point.id + 2,
            pointerType: "touch",
        });
        prepared?.(clickAccepted);
        if (clickAccepted && !prepared) activate(state.target);
    }

    function dispatchTouch(command) {
        command.modifiers = Number(command.modifiers ?? 0);
        const points = Array.from(command.touchPoints ?? [], point => ({
            id: Number(point.id ?? 0),
            x: Number(point.x),
            y: Number(point.y),
            radiusX: Number(point.radiusX ?? 1),
            radiusY: Number(point.radiusY ?? 1),
            rotationAngle: Number(point.rotationAngle ?? 0),
            force: Number(point.force ?? 1),
            tangentialPressure: Number(point.tangentialPressure ?? 0),
        }));

        if (command.eventType === "touchStart") {
            const started = [];
            for (const point of points) {
                const existing = activeTouches.get(point.id);
                if (existing) {
                    existing.point = point;
                    existing.touch = touchFromPoint(point, existing.target);
                    continue;
                }
                const target = document.elementFromPoint(point.x, point.y);
                if (!target) continue;
                const state = {
                    target,
                    point,
                    startX: point.x,
                    startY: point.y,
                    moved: false,
                    startAccepted: true,
                };
                state.touch = touchFromPoint(point, target);
                activeTouches.set(point.id, state);
                touchPointer(target, "pointerover", point, command.modifiers, 1);
                touchPointer(target, "pointerenter", point, command.modifiers, 1);
                touchPointer(target, "pointerdown", point, command.modifiers, 1);
                started.push(state);
            }
            const accepted = dispatchTouchToTargets("touchstart", started, command.modifiers);
            for (const state of started) state.startAccepted = accepted;
            return true;
        }

        if (command.eventType === "touchMove") {
            const moved = [];
            for (const point of points) {
                const state = activeTouches.get(point.id);
                if (!state) continue;
                state.point = point;
                state.touch = touchFromPoint(point, state.target);
                state.moved ||= Math.hypot(point.x - state.startX, point.y - state.startY) > 10;
                touchPointer(state.target, "pointermove", point, command.modifiers, 1);
                moved.push(state);
            }
            dispatchTouchToTargets("touchmove", moved, command.modifiers);
            return true;
        }

        if (command.eventType === "touchEnd" || command.eventType === "touchCancel") {
            const ending = points.length
                ? points.map(point => activeTouches.get(point.id)).filter(Boolean)
                : Array.from(activeTouches.values());
            for (const state of ending) activeTouches.delete(state.point.id);
            const accepted = dispatchTouchToTargets(
                command.eventType === "touchEnd" ? "touchend" : "touchcancel",
                ending,
                command.modifiers,
            );
            if (command.eventType === "touchEnd" && ending.length > 0) __notifyUserActivation();
            for (const state of ending) {
                const pointerType = command.eventType === "touchEnd" ? "pointerup" : "pointercancel";
                touchPointer(state.target, pointerType, state.point, command.modifiers, 0);
                touchPointer(state.target, "pointerout", state.point, command.modifiers, 0);
                touchPointer(state.target, "pointerleave", state.point, command.modifiers, 0);
            }
            if (command.eventType === "touchEnd" && ending.length === 1) {
                const state = ending[0];
                if (state.startAccepted && accepted && !state.moved) compatibilityClick(state, command.modifiers);
            }
            return true;
        }
        throw new TypeError(`unsupported touch event type: ${command.eventType}`);
    }

    function targetForSelector(selector) {
        const target = document.querySelector(String(selector));
        if (!target) throw new Error(`BRIMP_INPUT_NOT_FOUND:${selector}`);
        return target;
    }

    function center(target) {
        target.scrollIntoView({ block: "center", inline: "center" });
        const rect = target.getBoundingClientRect();
        return { x: rect.left + rect.width / 2, y: rect.top + rect.height / 2 };
    }

    function clickElement(target, clickCount, modifiers) {
        const point = moveElement(target, clickCount, modifiers);
        dispatchMouse({ action: "mouse", eventType: "mousePressed", ...point, button: 0, buttons: 1, clickCount, modifiers });
        dispatchMouse({ action: "mouse", eventType: "mouseReleased", ...point, button: 0, buttons: 0, clickCount, modifiers });
    }

    function moveElement(target, clickCount, modifiers) {
        const point = center(target);
        dispatchMouse({ action: "mouse", eventType: "mouseMoved", ...point, button: 0, buttons: 0, clickCount, modifiers });
        return point;
    }

    function typeText(target, text) {
        focus(target);
        for (const character of Array.from(String(text))) {
            const command = { eventType: "keyDown", key: character, code: "", text: character, modifiers: 0 };
            dispatchKey(command);
            dispatchKey({ ...command, eventType: "keyUp", text: "" });
        }
    }

    function tapElement(target, modifiers) {
        const point = center(target);
        const touchPoints = [{ id: 0, ...point, radiusX: 1, radiusY: 1, force: 1 }];
        dispatchTouch({ eventType: "touchStart", touchPoints, modifiers });
        dispatchTouch({ eventType: "touchEnd", touchPoints: [], modifiers });
    }

    return (serialized, targetOverride) => {
        const command = JSON.parse(String(serialized));
        let result;
        if (command.action === "mouse") result = dispatchMouse(command);
        else if (command.action === "key") result = dispatchKey(command);
        else if (command.action === "touch") result = dispatchTouch(command);
        else if (command.action === "insertText") result = insertText(command.text);
        else if (command.action === "click") result = clickElement(targetForSelector(command.selector), command.clickCount ?? 1, command.modifiers ?? 0);
        else if (command.action === "hover") result = moveElement(targetForSelector(command.selector), 0, command.modifiers ?? 0);
        else if (command.action === "type") result = typeText(targetForSelector(command.selector), command.text);
        else if (command.action === "tap") result = tapElement(targetForSelector(command.selector), command.modifiers ?? 0);
        else if (command.action === "focusTarget") result = focus(targetOverride);
        else throw new TypeError(`unsupported input action: ${command.action}`);
        return JSON.stringify(result ?? true);
    };
})();
