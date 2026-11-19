globalThis.__brimpInputController = (() => {
  const apply = Reflect.apply;
  const dispatchEvent = EventTarget.prototype.dispatchEvent;
  let mouseTarget = null;
  let mouseDownTarget = null;
  let mouseButtons = 0;
  let drag = null;
  // Where the pointer is. A browser reports it on every mouse and pointer event,
  // and sensors read it, so the position is tracked even when a command targets an
  // element rather than a coordinate.
  let mouseX = 0;
  let mouseY = 0;
  let lastScreenX = 0;
  let lastScreenY = 0;

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
    const clientX = overrides.clientX ?? command.x ?? mouseX;
    const clientY = overrides.clientY ?? command.y ?? mouseY;
    return {
      clientX,
      clientY,
      screenX: overrides.screenX ?? command.screenX ?? Math.round(clientX + (window.screenX || 0)),
      screenY: overrides.screenY ?? command.screenY ?? Math.round(clientY + (window.screenY || 0)),
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
    return dispatch(
      target,
      new PointerEvent(
        type,
        mouseOptions(command, {
          pointerId: overrides.pointerId ?? 1,
          pointerType: overrides.pointerType ?? command.pointerType ?? 'mouse',
          isPrimary: true,
          pressure: command.force ?? (command.buttons ? 0.5 : 0),
          tangentialPressure: command.tangentialPressure ?? 0,
          tiltX: command.tiltX ?? 0,
          tiltY: command.tiltY ?? 0,
          twist: command.twist ?? 0,
          ...overrides,
        }),
      ),
    );
  }

  function mouse(target, type, command, overrides = {}) {
    return dispatch(target, new MouseEvent(type, mouseOptions(command, overrides)));
  }

  function enterTarget(target, command) {
    if (target === mouseTarget) return;
    if (mouseTarget) {
      pointer(mouseTarget, 'pointerout', command, { relatedTarget: target });
      mouse(mouseTarget, 'mouseout', command, { relatedTarget: target });
      pointer(mouseTarget, 'pointerleave', command, {
        bubbles: false,
        relatedTarget: target,
      });
      mouse(mouseTarget, 'mouseleave', command, { bubbles: false, relatedTarget: target });
    }
    if (target) {
      pointer(target, 'pointerover', command, { relatedTarget: mouseTarget });
      mouse(target, 'mouseover', command, { relatedTarget: mouseTarget });
      pointer(target, 'pointerenter', command, {
        bubbles: false,
        relatedTarget: mouseTarget,
      });
      mouse(target, 'mouseenter', command, { bubbles: false, relatedTarget: mouseTarget });
    }
    mouseTarget = target;
  }

  // The point a pointer lands on for an element: the centre of its border box,
  // like a click through the page.
  function pointFor(element) {
    const box = element.getBoundingClientRect();
    return {
      x: Math.round(box.x + box.width / 2),
      y: Math.round(box.y + box.height / 2),
    };
  }

  function elementAt(x, y) {
    const hit = document.elementFromPoint(x, y);
    return hit && hit instanceof Element ? hit : document.documentElement;
  }

  function focus(target) {
    if (!target || target.disabled || document.activeElement === target) return;
    if (
      target instanceof HTMLInputElement &&
      __inputTemporalType(__inputType(target)) &&
      __controlConnected(target)
    )
      __inputTemporalEditors.add(target);
    const previous = document.activeElement;
    if (previous) {
      dispatch(previous, new FocusEvent('blur', { relatedTarget: target }));
      dispatch(
        previous,
        new FocusEvent('focusout', {
          bubbles: true,
          composed: true,
          relatedTarget: target,
        }),
      );
    }
    __activeElement = target;
    dispatch(target, new FocusEvent('focus', { relatedTarget: previous }));
    dispatch(
      target,
      new FocusEvent('focusin', { bubbles: true, composed: true, relatedTarget: previous }),
    );
  }

  function focusableTarget(target) {
    return (
      target?.closest?.('input,textarea,select,button,a[href],[tabindex],[contenteditable]') ?? null
    );
  }

  function inputEvent(target, type = 'input') {
    dispatch(target, new Event(type, { bubbles: true, composed: true }));
  }

  function activationElement(target) {
    return target?.closest?.('button,input,label,a[href]') ?? target;
  }

  function activate(target) {
    target = activationElement(target);
    if (!target || target.disabled) return;
    if (target instanceof HTMLLabelElement) {
      const control = target.htmlFor
        ? document.getElementById(target.htmlFor)
        : target.querySelector('input,button,select,textarea');
      if (control) clickElement(control, 1, 0);
      return;
    }
    if (target instanceof HTMLInputElement) {
      if (target.type === 'submit') {
        submit(__formOwner(target));
      } else if (target.type === 'reset') {
        reset(__formOwner(target));
      }
    } else if (target instanceof HTMLButtonElement) {
      if (__buttonType(target) === 'submit') submit(__formOwner(target));
      else if (__buttonType(target) === 'reset') reset(__formOwner(target));
    } else if (target instanceof HTMLAnchorElement && target.href) {
      location.href = target.href;
    }
  }

  function submit(form) {
    if (form) __trustedActivation(() => form.requestSubmit());
  }

  function reset(form) {
    if (form) __trustedActivation(() => form.reset());
  }

  function releaseClick(target, command) {
    const activationTarget = activationElement(target);
    if (mouseDownTarget !== target || activationTarget?.disabled) return;
    if (command.button === 0) {
      const accepted = pointer(target, 'click', command);
      if (accepted) activate(target);
      if (command.clickCount === 2) mouse(target, 'dblclick', command);
    } else if (command.button === 1) {
      mouse(target, 'auxclick', command);
    } else if (command.button === 2) {
      mouse(target, 'contextmenu', command);
    }
  }

  function moveElement(target, clickCount = 0, modifiers = 0, point = null) {
    const at = point ?? pointFor(target);
    mouseX = at.x;
    mouseY = at.y;
    const command = { button: 0, buttons: mouseButtons, clickCount, modifiers, x: at.x, y: at.y };
    enterTarget(target, command);
    pointer(target, 'pointermove', command);
    mouse(target, 'mousemove', command);
    return command;
  }
  function clickElement(target, clickCount = 1, modifiers = 0, point = null) {
    const command = moveElement(target, clickCount, modifiers, point);
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

  const dragEffects = new Map([
    [0, 'none'],
    [1, 'copy'],
    [2, 'link'],
    [3, 'copyLink'],
    [16, 'move'],
    [17, 'copyMove'],
    [18, 'linkMove'],
    [19, 'all'],
  ]);
  function dragEvent(target, type, command, data) {
    if (!target) return true;
    return dispatch(
      target,
      new DragEvent(
        type,
        mouseOptions(command, {
          dataTransfer: data,
          cancelable: type !== 'dragleave' && type !== 'dragend',
        }),
      ),
    );
  }
  function startDrag(source, command) {
    if (!source || source.getAttribute('draggable') === 'false') return;
    const data = new DataTransfer();
    data.effectAllowed = 'all';
    if (source.href || source.src) {
      data.setData('text/uri-list', source.href || source.src);
      data.setData('text/plain', source.textContent || source.href || source.src);
    }
    if (!dragEvent(source, 'dragstart', command, data)) {
      mouseDownTarget = null;
      return;
    }
    data.__readOnly = true;
    drag = { source, target: null, data, accepted: false };
    pointer(source, 'pointercancel', command);
  }
  function updateDrag(target, command) {
    if (target !== drag.target) {
      dragEvent(drag.target, 'dragleave', command, drag.data);
      dragEvent(target, 'dragenter', command, drag.data);
      drag.target = target;
    }
    const effect = drag.data.effectAllowed;
    drag.data.dropEffect =
      effect.includes('copy') || effect === 'all' || effect === 'uninitialized'
        ? 'copy'
        : effect.includes('move')
          ? 'move'
          : effect.includes('link')
            ? 'link'
            : 'none';
    drag.accepted = !dragEvent(target, 'dragover', command, drag.data);
    if (target instanceof HTMLInputElement && target.type === 'file') drag.accepted = true;
  }
  function finishDrag(target, command, cancelled) {
    if (!drag) return;
    const mask =
      Array.from(dragEffects).find(([, effect]) => effect === drag.data.effectAllowed)?.[0] ?? 19;
    const operation = { copy: 1, link: 2, move: 16 }[drag.data.dropEffect] || 0;
    cancelled ||= !(mask & operation);
    if (!cancelled && drag.accepted && target && drag.data.dropEffect !== 'none') {
      const accepted = dragEvent(target, 'drop', command, drag.data);
      if (accepted && target instanceof HTMLInputElement && target.type === 'file') {
        const files = Array.from(drag.data.files);
        target.__setFiles(target.multiple ? files : files.slice(0, 1));
        inputEvent(target);
        inputEvent(target, 'change');
      }
    } else {
      drag.data.dropEffect = 'none';
      dragEvent(drag.target, 'dragleave', command, drag.data);
    }
    dragEvent(drag.source, 'dragend', command, drag.data);
    drag = null;
    mouseDownTarget = null;
    mouseButtons = 0;
  }
  function dragTo(source, target) {
    const command = moveElement(source);
    startDrag(source, command);
    if (drag) {
      updateDrag(target, command);
      finishDrag(target, command, false);
    }
  }

  function editableTarget() {
    const target = document.activeElement;
    if (!target) return null;
    if (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement) {
      if (__isDisabledFormControl(target) || __getContentAttribute(target, 'readonly') !== null)
        return null;
    } else if (target.disabled || target.readOnly) return null;
    if (target instanceof HTMLTextAreaElement) return target;
    if (
      target instanceof HTMLInputElement &&
      ['text', 'search', 'url', 'tel', 'email', 'password', 'number'].includes(__inputType(target))
    )
      return target;
    return target.closest?.('[contenteditable="true"],[contenteditable=""]') || null;
  }
  function selection(target) {
    if (target instanceof HTMLInputElement) {
      const value = __inputEditorValue(target),
        { start, end } = __controlSelection(target);
      return {
        value,
        start: Math.min(start, value.length),
        end: Math.min(end, value.length),
      };
    }
    if ('value' in target)
      return {
        value: String(target.value),
        start: target.selectionStart,
        end: target.selectionEnd,
      };
    const value = target.textContent;
    const range = document.getSelection()?.rangeCount
      ? document.getSelection().getRangeAt(0)
      : null;
    if (
      range &&
      range.startContainer === target.firstChild &&
      range.endContainer === target.firstChild
    )
      return { value, start: range.startOffset, end: range.endOffset };
    return { value, start: value.length, end: value.length };
  }
  function caret(target, start, end = start) {
    if (target instanceof HTMLInputElement) {
      if (__supportsSelection(target)) __setControlSelection(target, start, end, 'none');
      else {
        const length = __inputEditorValue(target).length;
        end = Math.min(end, length);
        start = Math.min(start, end);
        __controlSelections.set(target, { start, end, direction: 'none' });
      }
    } else if ('value' in target) target.setSelectionRange(start, end);
    else if (target.firstChild) {
      const range = document.createRange();
      range.setStart(target.firstChild, start);
      range.setEnd(target.firstChild, end);
      const selected = document.getSelection();
      selected.removeAllRanges();
      selected.addRange(range);
    }
  }
  function replaceText(target, text, inputType = 'insertText', start, end) {
    const selected = selection(target);
    start ??= selected.start;
    end ??= selected.end;
    if (
      !dispatch(
        target,
        new InputEvent('beforeinput', {
          bubbles: true,
          cancelable: true,
          composed: true,
          data: text || null,
          inputType,
        }),
      )
    )
      return false;
    if (target instanceof HTMLInputElement) {
      const type = __inputType(target);
      if (type === 'number') text = __filterInputNumberText(text, selected.value, start, end);
      else {
        text = text.replace(/[\r\n]/g, '');
        const max = __parseNonnegativeInteger(__getContentAttribute(target, 'maxlength') ?? '');
        if (max !== null) {
          const available = Math.max(0, max - (selected.value.length - (end - start)));
          let limited = '';
          for (const character of text) {
            if (limited.length + character.length > available) break;
            limited += character;
          }
          text = limited;
        }
      }
      if (text === '' && start === end) return false;
    }
    const value = selected.value.slice(0, start) + text + selected.value.slice(end);
    if (target instanceof HTMLInputElement) __setInputEditedValue(target, value);
    else if (target instanceof HTMLTextAreaElement) __inputValues.set(target, value);
    else if ('value' in target) target.value = value;
    else target.textContent = value;
    caret(target, start + text.length);
    dispatch(
      target,
      new InputEvent('input', {
        bubbles: true,
        composed: true,
        data: text || null,
        inputType,
      }),
    );
    return true;
  }
  function edit(target, key, text, modifiers) {
    if (!target) return;
    const { value, start, end } = selection(target);
    if (modifiers & 6 && key.toLowerCase() === 'a') {
      caret(target, 0, value.length);
      return;
    }
    if (modifiers & 6) return;
    if (key === 'ArrowLeft') {
      caret(
        target,
        Math.max(
          0,
          start - (start === end ? Array.from(value.slice(0, start)).pop()?.length || 1 : 0),
        ),
      );
      return;
    }
    if (key === 'ArrowRight') {
      caret(
        target,
        Math.min(
          value.length,
          end + (start === end ? Array.from(value.slice(end))[0]?.length || 1 : 0),
        ),
      );
      return;
    }
    if (key === 'Home') {
      caret(target, 0);
      return;
    }
    if (key === 'End') {
      caret(target, value.length);
      return;
    }
    if (key === 'Backspace')
      replaceText(
        target,
        '',
        'deleteContentBackward',
        start === end
          ? Math.max(0, start - (Array.from(value.slice(0, start)).pop()?.length || 1))
          : start,
        end,
      );
    else if (key === 'Delete')
      replaceText(
        target,
        '',
        'deleteContentForward',
        start,
        start === end
          ? Math.min(value.length, end + (Array.from(value.slice(end))[0]?.length || 1))
          : end,
      );
    else if (key === 'Enter' && !(target instanceof HTMLInputElement))
      replaceText(target, '\n', 'insertLineBreak');
    else if (text) replaceText(target, text);
  }

  function tab(command) {
    const controls = Array.from(
      document.querySelectorAll(
        'input,textarea,select,button,a[href],[tabindex],[contenteditable]',
      ),
    ).filter((element) => !element.disabled && element.getAttribute('tabindex') !== '-1');
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
    const type = {
      keyDown: 'keydown',
      rawKeyDown: 'keydown',
      keyUp: 'keyup',
      char: 'keypress',
    }[command.eventType];
    if (!type) throw new TypeError(`unsupported key event type: ${command.eventType}`);
    if (
      (command.eventType === 'keyDown' || command.eventType === 'rawKeyDown') &&
      command.key !== 'Escape'
    ) {
      __notifyUserActivation();
    }
    const event = new KeyboardEvent(type, {
      bubbles: true,
      cancelable: true,
      composed: true,
      key: command.key ?? '',
      code: command.code ?? '',
      repeat: Boolean(command.autoRepeat),
      location: command.isKeypad ? 3 : (command.location ?? 0),
      keyCode: command.windowsVirtualKeyCode ?? command.nativeVirtualKeyCode ?? 0,
      charCode: command.eventType === 'char' ? (command.text || '').codePointAt(0) || 0 : 0,
      ...modifierOptions(command.modifiers),
    });
    const accepted = dispatch(target, event);
    if (accepted && command.key === 'Escape') finishDrag(null, {}, true);
    if (accepted && ['keyDown', 'rawKeyDown', 'char'].includes(command.eventType)) {
      if (command.key === 'Tab') tab(command);
      else if (command.key === 'Enter' && target instanceof HTMLInputElement)
        submit(__formOwner(target));
      else if (command.commands?.length) {
        const keys = {
          selectAll: 'a',
          deleteBackward: 'Backspace',
          deleteForward: 'Delete',
          insertNewline: 'Enter',
          moveLeft: 'ArrowLeft',
          moveRight: 'ArrowRight',
          moveToBeginningOfLine: 'Home',
          moveToEndOfLine: 'End',
        };
        for (const name of command.commands)
          edit(editableTarget(), keys[name], '', name === 'selectAll' ? 2 : 0);
      } else {
        const text = command.eventType === 'rawKeyDown' ? '' : (command.text ?? '');
        let characterAccepted = true;
        if (text && command.eventType === 'keyDown')
          characterAccepted = dispatch(
            target,
            new KeyboardEvent('keypress', {
              bubbles: true,
              cancelable: true,
              composed: true,
              key: command.key,
              code: command.code,
              charCode: text.codePointAt(0),
              ...modifierOptions(command.modifiers),
            }),
          );
        if (characterAccepted) edit(editableTarget(), command.key ?? '', text, command.modifiers);
      }
    }
    return true;
  }

  function insertText(text) {
    const target = editableTarget();
    if (!target) throw new Error('BRIMP_INPUT_NO_FOCUS');
    replaceText(target, String(text));
    return true;
  }

  function tapElement(target, modifiers = 0) {
    const command = { button: 0, buttons: 1, clickCount: 1, modifiers, pointerType: 'touch' };
    const point = new Touch({ identifier: 0, target, force: 1 });
    pointer(target, 'pointerover', command);
    pointer(target, 'pointerenter', command, { bubbles: false });
    pointer(target, 'pointerdown', command);
    const accepted = dispatch(
      target,
      new TouchEvent('touchstart', {
        bubbles: true,
        cancelable: true,
        composed: true,
        touches: [point],
        targetTouches: [point],
        changedTouches: [point],
        ...modifierOptions(modifiers),
      }),
    );
    __notifyUserActivation();
    command.buttons = 0;
    pointer(target, 'pointerup', command);
    const ended = dispatch(
      target,
      new TouchEvent('touchend', {
        bubbles: true,
        cancelable: true,
        composed: true,
        touches: [],
        targetTouches: [],
        changedTouches: [point],
        ...modifierOptions(modifiers),
      }),
    );
    pointer(target, 'pointerout', command);
    pointer(target, 'pointerleave', command, { bubbles: false });
    if (accepted && ended && !target.disabled) {
      mouse(target, 'mouseover', command);
      mouse(target, 'mousemove', command);
      command.buttons = 1;
      if (mouse(target, 'mousedown', command)) focus(focusableTarget(target));
      command.buttons = 0;
      mouse(target, 'mouseup', command);
      mouseDownTarget = target;
      releaseClick(target, command);
      mouseDownTarget = null;
    }
  }

  // Pointer state for commands that arrive as coordinates rather than a selector.
  const buttonBits = { left: 1, right: 2, middle: 4 };
  const buttonIndex = { left: 0, right: 2, middle: 1 };

  function buttonOf(command) {
    const name = command.button ?? 'left';
    if (!(name in buttonBits)) throw new TypeError(`unsupported button: ${name}`);
    return name;
  }

  function pointOf(command) {
    if (typeof command.x !== 'number' || typeof command.y !== 'number')
      throw new TypeError('coordinate commands require x and y');
    return { x: Math.round(command.x), y: Math.round(command.y) };
  }

  // Move the pointer to a point, in `steps` increments. A sensor watches the
  // number and spacing of `pointermove` samples, so a single jump is not the same
  // as travel.
  function movePointer(command) {
    const to = pointOf(command);
    const steps = Math.max(1, Math.min(1000, Math.trunc(Number(command.steps ?? 1)) || 1));
    // A pointer already at the point does not move, so no event is reported.
    if (steps === 1 && to.x === mouseX && to.y === mouseY) return;
    const fromX = mouseX;
    const fromY = mouseY;
    for (let step = 1; step <= steps; step++) {
      const ratio = step / steps;
      const x = step === steps ? to.x : Math.round(fromX + (to.x - fromX) * ratio);
      const y = step === steps ? to.y : Math.round(fromY + (to.y - fromY) * ratio);
      const event = {
        button: -1,
        buttons: mouseButtons,
        clickCount: 0,
        modifiers: command.modifiers ?? 0,
        x,
        y,
      };
      const target = elementAt(x, y);
      mouseX = x;
      mouseY = y;
      enterTarget(target, event);
      pointer(target, 'pointermove', event);
      mouse(target, 'mousemove', event);
    }
  }

  function pressButton(command) {
    const name = buttonOf(command);
    const at = pointOf(command);
    const target = elementAt(at.x, at.y);
    movePointer({ ...command, steps: command.steps ?? 1 });
    const event = {
      button: buttonIndex[name],
      buttons: mouseButtons | buttonBits[name],
      clickCount: command.clickCount ?? 1,
      modifiers: command.modifiers ?? 0,
      x: at.x,
      y: at.y,
    };
    __notifyUserActivation();
    mouseDownTarget = target;
    mouseButtons = event.buttons;
    const accepted = pointer(target, 'pointerdown', event);
    if (accepted && mouse(target, 'mousedown', event)) focus(focusableTarget(target));
    return { target, event };
  }

  function releaseButton(command) {
    const name = buttonOf(command);
    const at = pointOf(command);
    const target = elementAt(at.x, at.y);
    const event = {
      button: buttonIndex[name],
      buttons: mouseButtons & ~buttonBits[name],
      clickCount: command.clickCount ?? 1,
      modifiers: command.modifiers ?? 0,
      x: at.x,
      y: at.y,
    };
    mouseButtons = event.buttons;
    pointer(target, 'pointerup', event);
    if (command.button === undefined || name === 'left' || mouseDownTarget !== null)
      mouse(target, 'mouseup', event);
    const downTarget = mouseDownTarget;
    mouseDownTarget = null;
    if (downTarget === target) {
      // A click belongs to the element the press landed on.
      const clickEvent = { ...event, buttons: 0 };
      pointer(target, 'click', clickEvent);
      if (buttonIndex[name] === 2) mouse(target, 'contextmenu', clickEvent);
      else if (buttonIndex[name] === 1) mouse(target, 'auxclick', clickEvent);
    }
  }

  function wheel(command) {
    const at = pointOf(command);
    const target = elementAt(at.x, at.y);
    const event = {
      button: -1,
      buttons: mouseButtons,
      clickCount: 0,
      modifiers: command.modifiers ?? 0,
      x: at.x,
      y: at.y,
    };
    mouseX = at.x;
    mouseY = at.y;
    enterTarget(target, event);
    const deltaX = Number(command.deltaX ?? 0);
    const deltaY = Number(command.deltaY ?? 0);
    if (deltaX === 0 && deltaY === 0) throw new TypeError('wheel requires deltaX or deltaY');
    dispatch(
      target,
      new WheelEvent('wheel', {
        ...mouseOptions(event),
        deltaX,
        deltaY,
        deltaZ: 0,
        deltaMode: command.deltaMode ?? 0,
      }),
    );
    globalThis.scrollBy?.(deltaX, deltaY);
  }

  function targetForSelector(selector) {
    const target = document.querySelector(String(selector));
    if (!target) throw new Error(`BRIMP_INPUT_NOT_FOUND:${selector}`);
    return target;
  }

  function typeText(target, text) {
    focus(target);
    for (const character of Array.from(String(text))) {
      const command = {
        eventType: 'keyDown',
        key: character,
        code: '',
        text: character,
        modifiers: 0,
      };
      dispatchKey(command);
      dispatchKey({ ...command, eventType: 'keyUp', text: '' });
    }
  }

  function fill(target, text) {
    focus(target);
    if (editableTarget() !== target) throw new Error('element is not editable');
    if (replaceText(target, String(text), 'insertText', 0, selection(target).value.length))
      inputEvent(target, 'change');
  }
  function files(target, uploads) {
    if (!(target instanceof HTMLInputElement) || target.type !== 'file')
      throw new TypeError('target must be a file input');
    if (!target.multiple && uploads.length > 1)
      throw new TypeError('input does not accept multiple files');
    target.__setFiles(
      uploads.map(
        (file) =>
          new File([Uint8Array.from(file.bytes)], file.name, {
            type: file.mimeType ?? '',
            lastModified: file.lastModified ?? 0,
          }),
      ),
    );
    inputEvent(target);
    inputEvent(target, 'change');
  }
  return (serialized, targetOverride) => {
    const command = JSON.parse(String(serialized));
    // Coordinate commands resolve their own target from the hit test.
    if (command.action === 'move' || command.action === 'wheel') {
      if (command.action === 'move') movePointer(command);
      else wheel(command);
      return 'true';
    }
    if (command.action === 'down') {
      pressButton(command);
      return 'true';
    }
    if (command.action === 'up') {
      releaseButton(command);
      return 'true';
    }
    if (command.action === 'clickAt') {
      pressButton(command);
      releaseButton(command);
      return 'true';
    }
    const target =
      targetOverride ??
      (command.selector == null ? document.activeElement : targetForSelector(command.selector));
    if (!target || !(target instanceof Element)) throw new TypeError('input requires an element');
    if (command.action === 'click')
      clickElement(target, command.clickCount ?? 1, command.modifiers ?? 0);
    else if (command.action === 'hover') moveElement(target, 0, command.modifiers ?? 0);
    else if (command.action === 'type') typeText(target, command.text);
    else if (command.action === 'tap') tapElement(target, command.modifiers ?? 0);
    else if (command.action === 'focus') focus(target);
    else if (command.action === 'fill') fill(target, command.text);
    else if (command.action === 'files') files(target, command.files);
    else if (command.action === 'dragTo') dragTo(target, targetForSelector(command.destination));
    else if (command.action === 'key') {
      focus(target);
      dispatchKey(command);
    } else if (command.action === 'press') {
      focus(target);
      const down = {
        ...command,
        eventType: 'keyDown',
        text: Array.from(command.key).length === 1 ? command.key : '',
      };
      dispatchKey(down);
      dispatchKey({ ...down, eventType: 'keyUp', text: '' });
    } else throw new TypeError(`unsupported input action: ${command.action}`);
    return 'true';
  };
})();
