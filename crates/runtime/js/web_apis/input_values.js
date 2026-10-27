// Canonical input values and synchronous attribute-driven state transitions.
function __cloneFormControlState(source, clone, deep) {
  const pending = [[source, clone]];
  while (pending.length) {
    const [original, copy] = pending.pop();
    if (original instanceof HTMLInputElement || original instanceof HTMLTextAreaElement) {
      // Presence in the value/checked maps represents the dirty flag.
      // Copy raw state rather than assigning through a sanitizing setter.
      if (__inputValues.has(original)) __inputValues.set(copy, __inputValues.get(original));
      if (original instanceof HTMLInputElement) {
        if (__inputUserEdited.has(original)) __inputUserEdited.add(copy);
        if (__inputCheckedStates.has(original))
          __inputCheckedStates.set(copy, __inputCheckedStates.get(original));
        if (__inputIndeterminateStates.has(original))
          __inputIndeterminateStates.set(copy, __inputIndeterminateStates.get(original));
      }
      // Safari starts clones with a collapsed selection at the beginning.
      // File lists, custom validity and listeners belong to the original.
      __controlSelections.set(copy, { start: 0, end: 0, direction: 'none' });
    }
    if (!deep) continue;
    const children = __callHost('childNodes', original),
      copies = __callHost('childNodes', copy);
    for (let index = 0; index < children.length; index++)
      pending.push([children[index], copies[index]]);
    if (original instanceof HTMLTemplateElement) {
      pending.push([__callHost('templateContent', original), __callHost('templateContent', copy)]);
    }
  }
}

const __inputTypes = [
  'hidden',
  'text',
  'search',
  'tel',
  'url',
  'email',
  'password',
  'date',
  'time',
  'datetime-local',
  'number',
  'range',
  'color',
  'checkbox',
  'radio',
  'file',
  'submit',
  'image',
  'reset',
  'button',
];
function __textareaDefaultValue(input) {
  return __callHost('childNodes', input)
    .filter((node) => [3, 4].includes(__callHost('nodeType', node)))
    .map((node) => __callHost('textContent', node))
    .join('');
}
function __inputType(input) {
  const type = __asciiLowercase(__getContentAttribute(input, 'type') ?? 'text');
  return __inputTypes.includes(type) ? type : 'text';
}
function __inputValueMode(type) {
  if (['hidden', 'submit', 'image', 'reset', 'button'].includes(type)) return 'default';
  if (type === 'checkbox' || type === 'radio') return 'default/on';
  return type === 'file' ? 'filename' : 'value';
}
function __validInputDate(value) {
  const match = /^(\d{4,})-(\d{2})-(\d{2})$/.exec(value);
  if (!match || match[0] !== value) return false;
  const year = Number(match[1]),
    month = Number(match[2]),
    day = Number(match[3]);
  if (year < 1 || year > 275760 || month < 1 || month > 12 || day < 1) return false;
  const leap = year % 4 === 0 && (year % 100 !== 0 || year % 400 === 0);
  if (day > [31, leap ? 29 : 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][month - 1]) return false;
  // Safari limits dates to the ECMAScript Date range, even for string setters.
  return year !== 275760 || month < 9 || (month === 9 && day <= 13);
}
function __validInputTime(value) {
  return /^(?:[01]\d|2[0-3]):[0-5]\d(?::[0-5]\d(?:\.\d{1,3})?)?$/.exec(value)?.[0] === value;
}
function __sanitizeLocalDateTime(value) {
  const parts = value.split(/[T ]/);
  if (parts.length !== 2 || !__validInputDate(parts[0]) || !__validInputTime(parts[1])) return '';
  const date = parts[0].split('-'),
    time = parts[1].split(':');
  date[0] = String(Number(date[0])).padStart(4, '0');
  if (date.join('-') === '275760-09-13' && /[1-9]/.test(parts[1])) return '';
  if (time.length === 3) {
    time[2] = time[2].replace(/(\.\d*?)0+$/, '$1').replace(/\.$/, '');
    if (time[2] === '00') time.pop();
  }
  return date.join('-') + 'T' + time.join(':');
}
function __sanitizeRangeValue(input, value) {
  const attribute = (name) => __parseInputDecimal(__getContentAttribute(input, name) ?? '');
  const rawMin = attribute('min'),
    rawMax = attribute('max');
  const min = rawMin ?? __inputDecimal(0n);
  let max = rawMax ?? __inputDecimal(100n);
  if (__inputDecimalCompare(max, min) < 0) max = min;
  let number =
    __parseInputDecimal(value) ??
    __inputDecimalDivide(__inputDecimalAdd(min, max), __inputDecimal(2n));
  if (__inputDecimalCompare(number, min) < 0) number = min;
  if (__inputDecimalCompare(number, max) > 0) number = max;
  if (__asciiLowercase(__getContentAttribute(input, 'step') ?? '') !== 'any') {
    const rawStep = attribute('step'),
      step = rawStep?.c > 0n ? rawStep : __inputDecimal(1n);
    const base = rawMin ?? attribute('value') ?? __inputDecimal(0n);
    const count = __inputDecimalRound(
      __inputDecimalDivide(__inputDecimalAdd(number, base, true), step),
    );
    let rounded = __inputDecimalAdd(base, __inputDecimalMultiply(count, step));
    if (rounded && __inputDecimalCompare(rounded, max) > 0)
      rounded = __inputDecimalAdd(rounded, step, true);
    else if (rounded && __inputDecimalCompare(rounded, min) < 0)
      rounded = __inputDecimalAdd(rounded, step);
    if (
      rounded &&
      __inputDecimalCompare(rounded, min) >= 0 &&
      __inputDecimalCompare(rounded, max) <= 0
    )
      number = rounded;
  }
  return __serializeInputDecimal(number);
}
function __sanitizeInputValue(input, type, value) {
  if (['text', 'search', 'tel', 'password', 'email', 'url'].includes(type))
    value = value.replace(/[\r\n]/g, '');
  const trim = (text) => text.replace(/^[\t\n\f\r ]+|[\t\n\f\r ]+$/g, '');
  if (type === 'email')
    return __getContentAttribute(input, 'multiple') !== null
      ? value.split(',').map(trim).join(',')
      : trim(value);
  if (type === 'url') return trim(value);
  if (type === 'number' && value !== '' && !Number.isFinite(__numericValue(value))) return '';
  if (type === 'date') return __validInputDate(value) ? value : '';
  if (type === 'time') return __validInputTime(value) ? value : '';
  if (type === 'datetime-local') return __sanitizeLocalDateTime(value);
  if (type === 'range') return __sanitizeRangeValue(input, value);
  if (type === 'color')
    return __inputColor.sanitize(
      value,
      __getContentAttribute(input, 'alpha') !== null,
      __asciiLowercase(__getContentAttribute(input, 'colorspace') ?? '') === 'display-p3',
    );
  return value;
}
function __inputValue(input, type = __inputType(input)) {
  const mode = __inputValueMode(type);
  if (mode === 'filename') {
    const files = __fileListData.get(__inputFiles.get(input));
    return files?.length ? 'C:\\fakepath\\' + files[0].name : '';
  }
  const attribute = __getContentAttribute(input, 'value');
  if (mode === 'default') return attribute ?? '';
  if (mode === 'default/on') return attribute ?? 'on';
  return __inputValues.has(input)
    ? __inputValues.get(input)
    : __sanitizeInputValue(input, type, attribute ?? '');
}
function __inputMutationBefore(operation, args) {
  if (
    !['setAttribute', 'removeAttribute', 'setAttributeNS', 'removeAttributeNS'].includes(operation)
  )
    return null;
  const [input] = args;
  if (!(input instanceof HTMLInputElement)) return null;
  const namespaced = operation.endsWith('NS');
  if (namespaced && args[1]) return null;
  const name = namespaced ? args[2] : __asciiLowercase(args[1]);
  if (
    !['type', 'multiple', 'min', 'max', 'value', 'alpha', 'colorspace', 'maxlength'].includes(name)
  )
    return null;
  const oldType = __inputType(input),
    oldMode = __inputValueMode(oldType),
    value = __inputValue(input, oldType);
  return () => {
    const type = __inputType(input),
      mode = __inputValueMode(type);
    if (name === 'maxlength') {
      if (__inputValues.has(input)) {
        const sanitized = __sanitizeInputValue(input, type, value);
        if (sanitized !== value) {
          __inputScriptValue(input, value, sanitized);
          __inputValues.set(input, sanitized);
          __controlValueChanged(input, value, sanitized);
        }
      }
      return;
    }
    if (
      name !== 'type' &&
      name !== 'multiple' &&
      type !== 'range' &&
      !(type === 'color' && ['alpha', 'colorspace'].includes(name))
    ) {
      if (
        name === 'value' &&
        !__inputValues.has(input) &&
        __controlFocused(input) &&
        value !== __inputValue(input)
      ) {
        __controlSelections.set(input, { start: 0, end: 0, direction: 'forward' });
      }
      return;
    }
    if (oldType !== type) {
      __resetInputEditing(input);
      if (oldMode === 'value' && (mode === 'default' || mode === 'default/on') && value !== '') {
        __setContentAttribute(input, 'value', value);
      }
      if (
        (oldMode === 'default' || oldMode === 'default/on') &&
        (mode === 'value' || mode === 'filename')
      ) {
        const attribute = __getContentAttribute(input, 'value');
        if (attribute === null) __inputValues.delete(input);
        else __inputValues.set(input, attribute);
      }
      // Safari retains the pre-file dirty value, but discards selected files.
      if (oldMode === 'filename') __inputFiles.delete(input);
    }
    if (mode === 'value' && __inputValues.has(input)) {
      const previous = __inputValues.get(input),
        sanitized = __sanitizeInputValue(input, type, previous);
      if (previous !== sanitized) __inputScriptValue(input, previous, sanitized);
      __inputValues.set(input, sanitized);
    }
    if (oldType !== type) {
      const focused = __controlFocused(input);
      const oldSelection = __selectionTypes.includes(oldType),
        newSelection = __selectionTypes.includes(type);
      if ((oldSelection && newSelection) || (focused && ['email', 'number'].includes(type))) {
        __controlValueChanged(input, value, __inputValue(input, type));
      } else if (focused && newSelection) {
        const selection = __controlSelection(input);
        __setControlSelection(input, selection.start, selection.end, selection.direction, false);
      } else if (!focused) __controlSelections.delete(input);
    }
  };
}
