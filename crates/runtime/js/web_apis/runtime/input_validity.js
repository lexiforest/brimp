// Input constraints read canonical control state. Eligibility (willValidate)
// gates validation events, but does not suppress every individual validity flag.
const __inputPatternRegExp = RegExp;
const __inputPatternTest = Function.prototype.call.bind(RegExp.prototype.test);
const __inputEmailPattern = /^[a-zA-Z0-9.!#$%&'*+\/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]*[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]*[a-zA-Z0-9])?)*$/;
function __inputValidityFlags(input, flags) {
    const type = __inputType(input), value = __inputValue(input, type);
    const attribute = name => __getContentAttribute(input, name);
    const required = attribute('required') !== null;
    if (type === 'number') flags.badInput = __inputEditorValue(input) !== '' && __inputNumberEditorValue(__inputEditorValue(input)) === '';
    if (type === 'radio') {
        const group = __radioGroup(input);
        if (group !== null) {
            const peers = __ensureCheckedRoot(group.root).filter(peer => __sameRadioGroup(group, __radioGroup(peer)));
            flags.valueMissing = peers.some(peer => __getContentAttribute(peer, 'required') !== null) && !peers.some(__rawInputChecked);
        }
    } else if (required) {
        if (type === 'checkbox') flags.valueMissing = !__inputChecked(input);
        else if (type === 'file') flags.valueMissing = !__fileListData.get(__inputFiles.get(input))?.length;
        else if (['text','search','tel','url','email','password','number','date','time','datetime-local'].includes(type))
            flags.valueMissing = value === '' && attribute('readonly') === null && !__isDisabledFormControl(input);
    }
    if (value === '') return;
    if (type === 'email') {
        const values = attribute('multiple') !== null ? value.split(',') : [value];
        flags.typeMismatch = !values.every(value => __inputPatternTest(__inputEmailPattern, value));
    }
    if (type === 'url') { try { new URL(value); } catch { flags.typeMismatch = true; } }
    if (['text','search','tel','url','email','password'].includes(type)) {
        if (__inputUserEdited.has(input)) {
            const min = __parseNonnegativeInteger(attribute('minlength') ?? '');
            const max = __parseNonnegativeInteger(attribute('maxlength') ?? '');
            flags.tooShort = min !== null && value.length < min;
            flags.tooLong = max !== null && value.length > max;
        }
        const pattern = attribute('pattern');
        if (pattern !== null) {
            try {
                const expression = new __inputPatternRegExp(`^(?:${pattern})$`, 'v');
                const values = type === 'email' && attribute('multiple') !== null ? value.split(',') : [value];
                flags.patternMismatch = values.some(value => value !== '' && !__inputPatternTest(expression, value));
            }
            catch { /* Invalid patterns impose no constraint. */ }
        }
    }
    if (['number','range','date','time','datetime-local'].includes(type)) {
        const number = __inputStepValue(type, value);
        if (!number) return;
        const {min, max, base, step, numeric} = __inputStepParameters(input, type);
        flags.rangeUnderflow = __inputDecimalCompare(number, min) < 0;
        flags.rangeOverflow = __inputDecimalCompare(number, max) > 0;
        if (type === 'time' && __inputDecimalCompare(min, max) > 0)
            flags.rangeUnderflow = flags.rangeOverflow = flags.rangeUnderflow && flags.rangeOverflow;
        if (step) flags.stepMismatch = __inputSteppingMismatch(number, base, step, numeric);
    }
}
