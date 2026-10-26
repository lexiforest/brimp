// English messages measured in installed Safari. Message generation is a core
// value operation and does not display validation UI or change focus.
const __inputTemporalEditors = new WeakSet();
const __inputTemporalInvalid = new WeakSet();
function __inputTemporalType(type) { return ['date','time','datetime-local'].includes(type); }
function __inputValidationValueChanged(input, previous, value) {
    __inputTemporalInvalid.delete(input);
    if (!__inputTemporalType(__inputType(input)) || previous === value) return;
    // Safari computes the cached checkValidity result before clearing the old
    // temporal editor fields. Live flags/messages read the updated value.
    if (__inputTemporalEditors.has(input) && previous !== '' && value === '') __inputTemporalInvalid.add(input);
    __inputTemporalEditors.add(input);
}
function __inputValidationMutationBefore(operation,args) {
    if (!['setAttribute','removeAttribute','setAttributeNS','removeAttributeNS'].includes(operation)) return null;
    const [input] = args;
    if (!(input instanceof HTMLInputElement)) return null;
    const namespaced = operation.endsWith('NS');
    if (namespaced && args[1]) return null;
    const name = namespaced ? args[2] : __asciiLowercase(args[1]);
    if (!['type','value','min','max','step','pattern','required','disabled','readonly','maxlength','minlength','multiple'].includes(name)) return null;
    const type = __inputType(input), previous = __inputValue(input,type), oldAttribute = __getContentAttribute(input,name);
    return () => {
        if (oldAttribute === __getContentAttribute(input,name)) return;
        const nextType = __inputType(input);
        if (name === 'type') {
            if (type === nextType) return;
            __inputTemporalInvalid.delete(input);
            if (__inputTemporalType(nextType)) __inputTemporalEditors.add(input);
            else __inputTemporalEditors.delete(input);
        } else if (name === 'value' && !__inputValues.has(input)) {
            __inputValidationValueChanged(input,previous,__inputValue(input,nextType));
        } else __inputTemporalInvalid.delete(input);
    };
}
function __inputValidationBound(input, type, bound) {
    if (type === 'number' || type === 'range') return __serializeInputDecimal(bound);
    const number = Number(__serializeInputDecimal(bound));
    if (type === 'date') return __serializeInputDate(number);
    const time = __serializeInputTime(number,__inputTimePrecision(input));
    return type === 'time' ? time : __serializeInputDate(number) + 'T' + time;
}
function __inputValidationMessage(input) {
    if (!__willValidate(input)) return '';
    const custom = __customValidity.get(input);
    if (custom) return custom;
    const flags = __validityFlags(input), type = __inputType(input);
    if (flags.badInput) return 'Enter a number';
    if (flags.valueMissing) {
        if (type === 'checkbox') return 'Select this checkbox';
        if (type === 'radio') return 'Select one of these options';
        if (type === 'file') return 'Select a file';
        return 'Fill out this field';
    }
    if (flags.typeMismatch) return type === 'email' ? 'Enter an email address' : 'Enter a URL';
    if (flags.patternMismatch) {
        const title = (__getContentAttribute(input,'title') ?? '').replace(/[\t\n\f\r ]+/g,' ').replace(/^ | $/g,'');
        return 'Match the requested format' + (title ? ': ' + title : '');
    }
    if (flags.tooShort) return 'Use at least ' + __parseNonnegativeInteger(__getContentAttribute(input,'minlength')) + ' characters';
    if (flags.tooLong) {
        const max = __parseNonnegativeInteger(__getContentAttribute(input,'maxlength'));
        return max === 1 ? 'Use no more than one character' : 'Use no more than ' + max + ' characters';
    }
    if (['number','range','date','time','datetime-local'].includes(type)) {
        const number = __inputStepValue(type,__inputValue(input,type));
        if (!number) return '';
        const {min,max} = __inputStepParameters(input,type);
        // Safari's message checks the bounds independently even for a reversed
        // time interval, where validity/checkValidity accept its outer ranges.
        if (__inputDecimalCompare(number,min) < 0) return 'Value must be greater than or equal to ' + __inputValidationBound(input,type,min);
        if (__inputDecimalCompare(number,max) > 0) return 'Value must be less than or equal to ' + __inputValidationBound(input,type,max);
        if (flags.stepMismatch) return 'Enter a valid value';
    }
    return '';
}
