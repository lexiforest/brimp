// Input stepping is a core value operation. Work in the same finite decimal
// representation as range sanitization so decimal steps do not accumulate
// binary floating-point error. Temporal steps are integral milliseconds.
function __inputStepValue(type, value) {
    if (type === 'number' || type === 'range') return __parseInputDecimal(value);
    let number;
    if (type === 'date' && __validInputDate(value)) number = __inputDateMilliseconds(value);
    if (type === 'time' && __validInputTime(value)) number = __inputTimeMilliseconds(value);
    if (type === 'datetime-local') {
        const normalized = __sanitizeLocalDateTime(value);
        if (normalized) {
            const [date, time] = normalized.split('T');
            number = __inputDateMilliseconds(date) + __inputTimeMilliseconds(time);
        }
    }
    return Number.isFinite(number) ? __parseInputDecimal(String(number)) : null;
}
function __inputStepParameters(input, type) {
    const attribute = name => __inputStepValue(type, __getContentAttribute(input, name) ?? '');
    const rawMin = attribute('min'), rawMax = attribute('max');
    const numeric = type === 'number' || type === 'range';
    const bounds = type === 'number' ? [-Number.MAX_VALUE, Number.MAX_VALUE]
        : type === 'range' ? [0, 100] : type === 'time' ? [0, 86399999] : [-62135596800000, 8640000000000000];
    const min = rawMin ?? __parseInputDecimal(String(bounds[0]));
    let max = rawMax ?? __parseInputDecimal(String(bounds[1]));
    if (type === 'range' && __inputDecimalCompare(max, min) < 0) max = min;
    const base = rawMin ?? attribute('value') ?? __inputDecimal(0n);
    const text = __getContentAttribute(input, 'step') ?? '';
    if (__asciiLowercase(text) === 'any') return {min, max, base, step:null, numeric};
    let step = __parseInputDecimal(text);
    if (!step || step.c <= 0n) step = __inputDecimal(numeric || type === 'date' ? 1n : 60n);
    if (!numeric) {
        if (type === 'date') step = __inputDecimalRound(step);
        else step = __inputDecimalRound(__inputDecimalMultiply(step, __inputDecimal(1000n)));
        if (step.c < 1n) step = __inputDecimal(1n);
        if (type === 'date') step = __inputDecimalMultiply(step, __inputDecimal(86400000n));
    }
    return {min, max, base, step, numeric};
}
function __inputSteppingMismatch(value, base, step, numeric) {
    const absolute = value => value && {...value, c: value.c < 0n ? -value.c : value.c};
    const distance = absolute(__inputDecimalAdd(value, base, true));
    if (!distance || __inputDecimalCompare(__inputDecimalDivide(distance, __inputDecimal(9007199254740992n)), step) > 0) return false;
    const quotient = __inputDecimalRound(__inputDecimalDivide(distance, step));
    const remainder = absolute(__inputDecimalAdd(distance, __inputDecimalMultiply(step, quotient), true));
    const tolerance = numeric ? __inputDecimalDivide(step, __inputDecimal(16777216n)) : __inputDecimal(0n);
    return __inputDecimalCompare(remainder, tolerance) > 0 && __inputDecimalCompare(remainder, __inputDecimalAdd(step, tolerance, true)) < 0;
}
function __stepInput(input, argument, direction) {
    // Conversion may mutate the input; read its canonical state afterwards.
    let count = ((+argument >> 0) * direction) >> 0;
    const type = __inputType(input);
    if (!['number','range','date','time','datetime-local'].includes(type)) throw new DOMException('This control cannot be stepped', 'InvalidStateError');
    const {min, max, base, step, numeric} = __inputStepParameters(input, type);
    if (step === null) throw new DOMException('No allowed step', 'InvalidStateError');
    if (__inputDecimalCompare(min, max) > 0) return;
    // At extreme magnitudes the step can disappear in decimal arithmetic.
    if (__inputDecimalCompare(__inputDecimalAdd(base, step, true), base) === 0 || !__inputDecimalDivide(base, step)) return;
    const align = (value, rounding) => __inputDecimalAdd(base, __inputDecimalMultiply(rounding(__inputDecimalDivide(__inputDecimalAdd(value, base, true), step)), step));
    let ceiling = align(max, value => __inputDecimalIntegral(value, false));
    if (!ceiling) return;
    if (__inputDecimalCompare(ceiling, max) > 0) ceiling = __inputDecimalAdd(ceiling, step, true);
    if (__inputDecimalCompare(ceiling, min) < 0) return;
    const current = __inputStepValue(type, __inputValue(input, type)) ?? __inputDecimal(0n);
    const mismatch = __inputSteppingMismatch(current, base, step, numeric);
    let next = current;
    if (mismatch && count !== 0) {
        next = align(current, value => __inputDecimalIntegral(value, count > 0));
        count += count > 0 ? -1 : 1;
    }
    next = __inputDecimalAdd(next, __inputDecimalMultiply(step, __inputDecimal(BigInt(count))));
    if (!next) return;
    if (!mismatch && __inputDecimalCompare(next, __inputDecimal(1n, 21)) < 0) next = align(next, __inputDecimalRound);
    if (!next) return;
    if (__inputDecimalCompare(next, min) < 0) next = align(min, value => __inputDecimalIntegral(value, true));
    if (__inputDecimalCompare(next, max) > 0) next = ceiling;
    // Safari uses the remaining count after snapping for this direction check.
    if (count < 0 && __inputDecimalCompare(current, next) < 0 || count > 0 && __inputDecimalCompare(current, next) > 0) return;
    if (numeric) {
        const previous = __inputValue(input,type), value = __sanitizeInputValue(input,type,__serializeInputDecimal(next));
        __inputScriptValue(input,previous,value);
        __inputValues.set(input,value);
        __controlValueChanged(input,previous,value);
    }
    else __setInputNumericValue(input, type, Number(__serializeInputDecimal(next)));
}
Object.assign(HTMLInputElement.prototype, {
    stepUp(n = 1) { __stepInput(this, n, 1); },
    stepDown(n = 1) { __stepInput(this, n, -1); },
});
