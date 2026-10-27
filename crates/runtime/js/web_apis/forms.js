const __customValidity = new WeakMap(),
  __validityStates = new WeakMap(),
  __validityElements = new WeakMap();
const __formControlSelector = 'button,fieldset,input,object,output,select,textarea';
function __prepareInputActivation(event, path) {
  if (event.type !== 'click' || !(event instanceof MouseEvent)) return null;
  const target = (event.bubbles ? path : path.slice(0, 1)).find(
    (node) => node instanceof HTMLInputElement && ['checkbox', 'radio'].includes(__inputType(node)),
  );
  if (!target) return null;
  const checked = __inputChecked(target),
    originalType = __inputType(target);
  const indeterminate =
    originalType === 'checkbox' ? (__inputIndeterminateStates.get(target) ?? false) : false;
  const group = __radioGroup(target);
  const previousRadio =
    group &&
    __ensureCheckedRoot(group.root).find(
      (peer) =>
        peer !== target && __rawInputChecked(peer) && __sameRadioGroup(group, __radioGroup(peer)),
    );
  __setInputChecked(target, originalType === 'radio' || !checked);
  if (originalType === 'checkbox') __inputIndeterminateStates.set(target, false);
  return (accepted) => {
    const type = __inputType(target);
    if (!accepted) {
      if (type === 'checkbox') {
        __inputCheckedStates.set(target, checked);
        __inputIndeterminateStates.set(target, indeterminate);
      } else if (type === 'radio') {
        __setInputChecked(target, checked);
        if (previousRadio && __sameRadioGroup(__radioGroup(target), __radioGroup(previousRadio)))
          __setInputChecked(previousRadio, true);
      }
      return;
    }
    if (type !== 'checkbox' && (type !== 'radio' || checked || !__inputChecked(target))) return;
    if (!__nodeIsConnected(target)) return;
    Reflect.apply(__dispatchEvent, target, [
      __markTrustedEvent(new Event('input', { bubbles: true, composed: true })),
    ]);
    Reflect.apply(__dispatchEvent, target, [
      __markTrustedEvent(new Event('change', { bubbles: true })),
    ]);
  };
}
function __formControls(form, includeImages = false) {
  return __callHost('querySelectorAll', __nodeTreeRoot(form), __formControlSelector).filter(
    (e) =>
      __formOwner(e) === form &&
      (includeImages || !(e instanceof HTMLInputElement) || __inputType(e) !== 'image'),
  );
}
for (const cls of [
  HTMLInputElement,
  HTMLButtonElement,
  HTMLSelectElement,
  HTMLTextAreaElement,
  HTMLFieldSetElement,
  HTMLObjectElement,
  HTMLOutputElement,
]) {
  Object.defineProperty(
    cls.prototype,
    'form',
    Object.getOwnPropertyDescriptor(
      {
        get form() {
          return __formOwner(this);
        },
      },
      'form',
    ),
  );
}
Object.defineProperty(HTMLFormElement.prototype, 'elements', {
  configurable: true,
  get() {
    return __cachedCollection(this, 'elements', () =>
      __createHTMLCollection(() => __formControls(this), HTMLFormControlsCollection.prototype),
    );
  },
});
const __resettingForms = new WeakSet();
HTMLFormElement.prototype.reset = function () {
  if (__documentWindow(__callHost('ownerDocument', this)) === null || __resettingForms.has(this))
    return;
  __resettingForms.add(this);
  try {
    if (!this.dispatchEvent(new Event('reset', { bubbles: true, cancelable: true }))) return;
    for (const e of __formControls(this, true)) {
      const textControl = e instanceof HTMLInputElement || e instanceof HTMLTextAreaElement;
      const previous = textControl ? __controlValue(e) : '';
      __inputValues.delete(e);
      if (e instanceof HTMLInputElement) {
        __resetInputEditing(e);
        if (__inputTemporalType(__inputType(e))) __inputTemporalEditors.add(e);
        __resetInputChecked(e);
        const length = __controlValue(e).length;
        __controlSelections.set(e, { start: length, end: length, direction: 'none' });
      } else if (textControl) __controlValueChanged(e, previous, __controlValue(e));
      if (e instanceof HTMLInputElement && __inputType(e) === 'file') __clearInputFiles(e);
      if (e instanceof HTMLSelectElement)
        for (const option of e.querySelectorAll('option')) __optionSelectedStates.delete(option);
    }
  } finally {
    __resettingForms.delete(this);
  }
};
function __numericValue(value) {
  return /^-?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?$/.exec(value)?.[0] === value &&
    !value.endsWith('.') &&
    Number.isFinite(Number(value))
    ? Number(value)
    : NaN;
}
function __willValidate(e) {
  if ([HTMLFieldSetElement, HTMLOutputElement, HTMLObjectElement].some((c) => e instanceof c))
    return false;
  if (
    e instanceof HTMLInputElement &&
    ['hidden', 'image', 'reset', 'button'].includes(__inputType(e))
  )
    return false;
  if (e instanceof HTMLButtonElement && __buttonType(e) !== 'submit') return false;
  if (
    (e instanceof HTMLInputElement || e instanceof HTMLTextAreaElement) &&
    __getContentAttribute(e, 'readonly') !== null
  )
    return false;
  if (__isDisabledFormControl(e)) return false;
  for (let parent = __callHost('parentNode', e); parent; parent = __callHost('parentNode', parent))
    if (parent instanceof HTMLDataListElement) return false;
  return true;
}
function __validityFlags(e) {
  const flags = {
    valueMissing: false,
    typeMismatch: false,
    patternMismatch: false,
    tooLong: false,
    tooShort: false,
    rangeUnderflow: false,
    rangeOverflow: false,
    stepMismatch: false,
    badInput: false,
    customError: !!__customValidity.get(e),
  };
  // Buttons have no built-in value constraints, including when script adds
  // input-like properties such as required, pattern or min/max.
  if (e instanceof HTMLButtonElement) return flags;
  if (e instanceof HTMLInputElement) {
    __inputValidityFlags(e, flags);
    return flags;
  }
  if (e.required) flags.valueMissing = (e.value ?? '') === '';
  return flags;
}
const __validityToken = {};
class ValidityState {
  constructor(...args) {
    if (args[0] !== __validityToken) throw new TypeError('Illegal constructor');
    __validityElements.set(this, args[1]);
  }
}
for (const flag of [
  'valueMissing',
  'typeMismatch',
  'patternMismatch',
  'tooLong',
  'tooShort',
  'rangeUnderflow',
  'rangeOverflow',
  'stepMismatch',
  'badInput',
  'customError',
  'valid',
]) {
  Object.defineProperty(
    ValidityState.prototype,
    flag,
    Object.getOwnPropertyDescriptor(
      {
        get [flag]() {
          const element = __validityElements.get(this);
          if (!element) throw new TypeError('Illegal invocation');
          if (flag === 'customError') return !!__customValidity.get(element);
          const flags = __validityFlags(element);
          return flag === 'valid' ? !Object.values(flags).some(Boolean) : flags[flag];
        },
      },
      flag,
    ),
  );
}
function __getValidity(e) {
  if (!__validityStates.has(e)) __validityStates.set(e, new ValidityState(__validityToken, e));
  return __validityStates.get(e);
}
function __checkValidity(e) {
  if (
    !__willValidate(e) ||
    (!__inputTemporalInvalid.has(e) && !Object.values(__validityFlags(e)).some(Boolean))
  )
    return true;
  Reflect.apply(__dispatchEvent, e, [
    __markTrustedEvent(new Event('invalid', { cancelable: true })),
  ]);
  return false;
}
for (const cls of [
  HTMLInputElement,
  HTMLButtonElement,
  HTMLSelectElement,
  HTMLTextAreaElement,
  HTMLFieldSetElement,
  HTMLObjectElement,
  HTMLOutputElement,
]) {
  Object.defineProperties(cls.prototype, {
    validity: Object.getOwnPropertyDescriptor(
      {
        get validity() {
          return __getValidity(this);
        },
      },
      'validity',
    ),
    willValidate: Object.getOwnPropertyDescriptor(
      {
        get willValidate() {
          return __willValidate(this);
        },
      },
      'willValidate',
    ),
    validationMessage: Object.getOwnPropertyDescriptor(
      {
        get validationMessage() {
          if (this instanceof HTMLInputElement) return __inputValidationMessage(this);
          return !__willValidate(this) || !Object.values(__validityFlags(this)).some(Boolean)
            ? ''
            : __customValidity.get(this) || 'The value does not satisfy the constraints.';
        },
      },
      'validationMessage',
    ),
  });
  Object.assign(cls.prototype, {
    setCustomValidity(message) {
      if (!arguments.length) throw new TypeError('A message is required');
      const value = `${message}`;
      __customValidity.set(this, value);
      __inputTemporalInvalid.delete(this);
    },
    checkValidity() {
      return __checkValidity(this);
    },
    // Preserve the lightweight validation result and events. The legacy
    // no-op presentation behavior supplies no browser popup or scrolling.
    reportValidity() {
      return __checkValidity(this);
    },
  });
}
HTMLFormElement.prototype.checkValidity = function () {
  let valid = true;
  for (const e of this.elements)
    if (typeof e.checkValidity === 'function' && !e.checkValidity()) valid = false;
  return valid;
};
HTMLFormElement.prototype.reportValidity = function () {
  return this.checkValidity();
};
