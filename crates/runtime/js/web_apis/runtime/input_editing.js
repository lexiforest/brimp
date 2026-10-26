// User edits have a visible buffer independent of the sanitized DOM value.
// In particular, incomplete numbers expose value="" without losing the text
// needed by the next keystroke or by ValidityState.badInput.
const __inputEditorValues = new WeakMap();
const __inputUserEdited = new WeakSet();
function __inputEditorValue(input) { return __inputEditorValues.get(input) ?? __inputValue(input); }
function __resetInputEditing(input) {
    __inputEditorValues.delete(input);
    __inputUserEdited.delete(input);
    __inputTemporalInvalid.delete(input);
}
function __inputScriptValue(input, previous, value) {
    __inputValidationValueChanged(input,previous,value);
    __inputUserEdited.delete(input);
    const editor = __inputEditorValues.get(input);
    __inputEditorValues.delete(input);
    if (editor !== undefined && editor !== value && previous === value && !__supportsSelection(input))
        __controlSelections.set(input,{start:0,end:0,direction:'none'});
}
function __inputNumberEditorValue(text) {
    // Decimal editing accepts a leading plus and a trailing decimal separator;
    // scientific notation retains its spelling and uses the strict parser.
    if (!/[eE]/.test(text)) text = text.replace(/^\+/, '').replace(/\.$/, '');
    return Number.isFinite(__numericValue(text)) ? text : '';
}
function __filterInputNumberText(text, value, start, end) {
    let left = value.slice(0,start);
    const right = value.slice(end);
    let accepted = '';
    for (let character of text) {
        const code = character.charCodeAt(0);
        if (code >= 0xff10 && code <= 0xff19) character = String(code - 0xff10);
        if (character === '\uff0d' || character === '\u2212') character = '-';
        if (character === '\uff0b') character = '+';
        if (character === '\uff0e') character = '.';
        if (!/[0-9.eE+-]/.test(character)) continue;
        const candidate = left + character + right;
        // This grammar admits unfinished mantissas and exponents, while
        // rejecting misplaced signs, duplicate separators and exponents.
        if (!/^[+-]?\d*(?:\.\d*)?(?:[eE][+-]?\d*)?$/.test(candidate)) continue;
        if (candidate.startsWith('+') && /[eE]/.test(candidate)) continue;
        accepted += character;
        left += character;
    }
    return accepted;
}
function __setInputEditedValue(input, text) {
    const type = __inputType(input);
    __inputEditorValues.set(input,text);
    __inputValues.set(input,type === 'number' ? __inputNumberEditorValue(text)
        : type === 'url' ? text : __sanitizeInputValue(input,type,text));
    __inputUserEdited.add(input);
}
