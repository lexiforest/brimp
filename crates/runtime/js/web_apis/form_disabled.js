// Safari stores ancestor-disabled state. Attribute changes on an already
// disabled nested fieldset can leave its first-legend controls exempt until a
// later tree mutation refreshes them. Keep this state private and shared by
// validation eligibility, script clicks, selection and form-data construction.
const __fieldsetDisabledStates = new WeakMap();
const __disabledControlSelector = 'input,button,select,textarea,fieldset,object,output';
function __hasDisabledAttribute(control) {
  return __getContentAttribute(control, 'disabled') !== null;
}
function __firstFieldsetLegend(fieldset) {
  let child = __callHost('firstChild', fieldset);
  while (child && !(child instanceof HTMLLegendElement)) child = __callHost('nextSibling', child);
  return child;
}
function __computeFieldsetDisabled(control) {
  for (
    let descendant = control, parent = __callHost('parentNode', control);
    parent;
    descendant = parent, parent = __callHost('parentNode', parent)
  ) {
    if (parent instanceof HTMLFieldSetElement && __hasDisabledAttribute(parent))
      return __firstFieldsetLegend(parent) !== descendant;
  }
  return false;
}
function __fieldsetDisabled(control) {
  if (!__fieldsetDisabledStates.has(control))
    __fieldsetDisabledStates.set(control, __computeFieldsetDisabled(control));
  return __fieldsetDisabledStates.get(control);
}
function __isDisabledFormControl(control) {
  return __hasDisabledAttribute(control) || __fieldsetDisabled(control);
}
function __isValidatedControl(node) {
  return [
    HTMLInputElement,
    HTMLButtonElement,
    HTMLSelectElement,
    HTMLTextAreaElement,
    HTMLFieldSetElement,
    HTMLObjectElement,
    HTMLOutputElement,
  ].some((type) => node instanceof type);
}
function __disabledControlsUnder(root) {
  const controls = __isValidatedControl(root) ? [root] : [];
  if (root instanceof Element || root instanceof Document || root instanceof DocumentFragment)
    controls.push(
      ...__callHost('querySelectorAll', root, __disabledControlSelector).filter(
        __isValidatedControl,
      ),
    );
  return controls;
}
function __setFieldsetDisabled(control, disabled) {
  const before = __hasDisabledAttribute(control) || __fieldsetDisabled(control);
  __fieldsetDisabledStates.set(control, disabled);
  if (before !== (__hasDisabledAttribute(control) || disabled))
    __inputTemporalInvalid.delete(control);
  if (
    control instanceof HTMLFieldSetElement &&
    !disabled &&
    before !== __hasDisabledAttribute(control)
  )
    __propagateFieldsetDisabled(control);
}
function __setDisabledSubtree(root, disabled) {
  if (__isValidatedControl(root)) __setFieldsetDisabled(root, disabled);
  if (root instanceof HTMLFieldSetElement && __hasDisabledAttribute(root)) return;
  for (let child = __callHost('firstChild', root); child; child = __callHost('nextSibling', child))
    if (child instanceof Element) __setDisabledSubtree(child, disabled);
}
function __propagateFieldsetDisabled(fieldset) {
  const legend = __firstFieldsetLegend(fieldset),
    disabled = __hasDisabledAttribute(fieldset);
  for (
    let child = __callHost('firstChild', fieldset);
    child;
    child = __callHost('nextSibling', child)
  )
    if (child instanceof HTMLElement)
      __setDisabledSubtree(child, child === legend ? false : disabled);
}
function __fieldsetChildrenChanged(fieldset) {
  if (!__hasDisabledAttribute(fieldset)) return;
  let first = true;
  for (
    let child = __callHost('firstChild', fieldset);
    child;
    child = __callHost('nextSibling', child)
  ) {
    if (!(child instanceof HTMLLegendElement)) continue;
    __setDisabledSubtree(child, !first);
    first = false;
  }
}
function __disabledMutationBefore(operation, args) {
  const [target, child, reference] = args;
  if (operation.includes('Attribute')) {
    if (!(target instanceof HTMLFieldSetElement)) return null;
    const attribute = String(operation.endsWith('NS') ? reference : child)
      .split(':')
      .at(-1);
    if (__asciiLowercase(attribute) !== 'disabled') return null;
    const own = __hasDisabledAttribute(target),
      ancestor = __fieldsetDisabled(target);
    // Capture controls before changing the attribute, even if no script has
    // read their disabled state yet.
    for (const control of __disabledControlsUnder(target)) __fieldsetDisabled(control);
    return () => {
      const next = __hasDisabledAttribute(target);
      if (own !== next && !ancestor) __propagateFieldsetDisabled(target);
    };
  }
  if (
    ![
      'appendChild',
      'insertBefore',
      'removeChild',
      'replaceChild',
      'setInnerHTML',
      'setTextContent',
    ].includes(operation)
  )
    return null;
  const replaceAll = operation === 'setInnerHTML' || operation === 'setTextContent';
  const parent = target instanceof HTMLTemplateElement ? target.content : target;
  const roots = replaceAll
    ? __callHost('childNodes', parent)
    : operation === 'replaceChild'
      ? [child, reference]
      : [child];
  const parents = new Set(parent instanceof HTMLFieldSetElement ? [parent] : []);
  const controls = new Set(),
    inserted = new Set(),
    validationCandidates = new Map();
  for (const root of roots) {
    if (!(root instanceof Node)) continue;
    const oldParent = __callHost('parentNode', root);
    if (oldParent instanceof HTMLFieldSetElement) parents.add(oldParent);
    for (const control of __disabledControlsUnder(root)) {
      __fieldsetDisabled(control);
      controls.add(control);
      if (__inputTemporalInvalid.has(control))
        validationCandidates.set(control, __willValidate(control));
      if (!replaceAll && operation !== 'removeChild' && root === child) inserted.add(control);
    }
  }
  return () => {
    if (replaceAll)
      for (const root of __callHost('childNodes', parent))
        for (const control of __disabledControlsUnder(root)) {
          controls.add(control);
          inserted.add(control);
        }
    // Removal refreshes a previously disabled control, but preserves an
    // exemption. Insertion always recomputes the ancestor state.
    for (const control of controls)
      if (inserted.has(control) || __fieldsetDisabled(control))
        __setFieldsetDisabled(control, __computeFieldsetDisabled(control));
    for (const fieldset of parents) __fieldsetChildrenChanged(fieldset);
    for (const [control, eligible] of validationCandidates)
      if (__willValidate(control) !== eligible) __inputTemporalInvalid.delete(control);
  };
}
