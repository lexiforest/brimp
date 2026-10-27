// Capture Date intrinsics: input conversion is independent of overridden page
// methods. Safari's legacy valueAsDate accepts numbers, strings and real Dates.
const __InputDate = Date;
const __inputDateParse = Date.parse;
const __inputDateTime = Function.prototype.call.bind(Date.prototype.getTime);
const __inputDateYear = Function.prototype.call.bind(Date.prototype.getUTCFullYear);
const __inputDateMonth = Function.prototype.call.bind(Date.prototype.getUTCMonth);
const __inputDateDay = Function.prototype.call.bind(Date.prototype.getUTCDate);
const __inputDateSetYear = Function.prototype.call.bind(Date.prototype.setUTCFullYear);
const __inputDayMilliseconds = 86400000;
function __inputDateMilliseconds(value) {
  const [year, month, day] = value.split('-').map(Number),
    date = new __InputDate(0);
  return __inputDateSetYear(date, year, month - 1, day);
}
function __inputTimeMilliseconds(value) {
  const [hours, minutes, seconds = '0'] = value.split(':');
  const [whole, fraction = ''] = seconds.split('.');
  return (
    Number(hours) * 3600000 +
    Number(minutes) * 60000 +
    Number(whole) * 1000 +
    Number(fraction.padEnd(3, '0'))
  );
}
function __inputNumber(input, type = __inputType(input)) {
  if (!['number', 'range', 'date', 'time', 'datetime-local'].includes(type)) return NaN;
  const value = __inputValue(input, type);
  if (!value) return NaN;
  if (type === 'number' || type === 'range') {
    const number = __numericValue(value);
    return number === 0 ? 0 : number;
  }
  if (type === 'date') return __inputDateMilliseconds(value);
  if (type === 'time') return __inputTimeMilliseconds(value);
  if (type === 'datetime-local') {
    const [date, time] = value.split('T');
    return __inputDateMilliseconds(date) + __inputTimeMilliseconds(time);
  }
  return NaN;
}
function __serializeInputDate(number) {
  const date = new __InputDate(
    Math.floor(number / __inputDayMilliseconds) * __inputDayMilliseconds,
  );
  const year = __inputDateYear(date);
  if (!(year >= 1)) return null;
  return (
    String(year).padStart(4, '0') +
    '-' +
    String(__inputDateMonth(date) + 1).padStart(2, '0') +
    '-' +
    String(__inputDateDay(date)).padStart(2, '0')
  );
}
// WebKit scales and rounds step to integer milliseconds before choosing the
// seconds/fraction format; use the same finite-precision decimal remainder.
function __inputTimePrecision(input) {
  const raw = __parseInputDecimal(__getContentAttribute(input, 'step') ?? '');
  if (!raw || raw.c <= 0n) return 0;
  let step = __inputDecimalRound(__inputDecimalMultiply(raw, __inputDecimal(1000n)));
  if (!step) return 0;
  if (step.c === 0n) step = __inputDecimal(1n);
  return __inputDecimalRemainder(step, __inputDecimal(60000n))?.c === 0n
    ? 0
    : __inputDecimalRemainder(step, __inputDecimal(1000n))?.c === 0n
      ? 1
      : 2;
}
function __serializeInputTime(number, precision = 0) {
  let value = number % __inputDayMilliseconds;
  if (value < 0) value += __inputDayMilliseconds;
  const hours = Math.floor(value / 3600000),
    minutes = Math.floor(value / 60000) % 60;
  const seconds = Math.floor(value / 1000) % 60,
    milliseconds = value % 1000;
  let result = String(hours).padStart(2, '0') + ':' + String(minutes).padStart(2, '0');
  if (seconds || milliseconds || precision) result += ':' + String(seconds).padStart(2, '0');
  if (milliseconds || precision === 2)
    result += '.' + (String(milliseconds).padStart(3, '0').replace(/0+$/, '') || '0');
  return result;
}
function __setInputNumericValue(input, type, number) {
  let value = null;
  if (Number.isFinite(number)) {
    if (type === 'number' || type === 'range')
      value = __sanitizeInputValue(input, type, String(number));
    else {
      // Temporal input serialization passes through WebKit Decimal's
      // double conversion before rounding milliseconds away from zero.
      number = Number(__serializeInputDecimal(__parseInputDecimal(String(number))));
      const rounded = number < 0 ? -Math.round(-number) : Math.round(number);
      if (type === 'date') value = __serializeInputDate(rounded);
      else if (type === 'time') value = __serializeInputTime(rounded, __inputTimePrecision(input));
      else if (rounded >= -62135596800000 && rounded <= 8640000000000000) {
        const date = __serializeInputDate(rounded);
        if (date !== null) value = date + 'T' + __serializeInputTime(rounded);
      }
    }
  }
  // Safari's null serialization restores the attribute-backed value rather
  // than installing a dirty empty string.
  const previous = __inputValue(input, type);
  if (value === null) __inputValues.delete(input);
  else __inputValues.set(input, value);
  const next = __inputValue(input, type);
  __inputScriptValue(input, previous, next);
  __controlValueChanged(input, previous, next);
}
Object.defineProperties(HTMLInputElement.prototype, {
  valueAsNumber: Object.getOwnPropertyDescriptor(
    {
      get valueAsNumber() {
        return __inputNumber(this);
      },
      set valueAsNumber(value) {
        const number = +value;
        if (!Number.isFinite(number))
          throw new DOMException('The value must be finite', 'NotSupportedError');
        const type = __inputType(this);
        if (!['number', 'range', 'date', 'time', 'datetime-local'].includes(type))
          throw new DOMException('This input type has no numeric value', 'InvalidStateError');
        __setInputNumericValue(this, type, number);
      },
    },
    'valueAsNumber',
  ),
  valueAsDate: Object.getOwnPropertyDescriptor(
    {
      get valueAsDate() {
        const type = __inputType(this);
        if (type !== 'date' && type !== 'time') return null;
        const number = __inputNumber(this, type);
        return Number.isNaN(number) ? null : new __InputDate(number);
      },
      set valueAsDate(value) {
        const type = __inputType(this);
        if (type !== 'date' && type !== 'time')
          throw new DOMException('This input type has no Date value', 'InvalidStateError');
        let number = NaN;
        if (typeof value === 'number') number = value;
        else if (typeof value === 'string') number = __inputDateParse(value);
        else {
          try {
            number = __inputDateTime(value);
          } catch {
            /* Non-Date objects clear the dirty value. */
          }
        }
        __setInputNumericValue(this, type, number);
      },
    },
    'valueAsDate',
  ),
});
