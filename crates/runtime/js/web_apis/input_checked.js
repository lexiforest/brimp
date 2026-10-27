// Presence in __inputCheckedStates is Safari's dirty checkedness flag. Radio
// deselection also dirties the peer; default attributes remain independent.
const __checkedInitialized = new WeakSet();
const __checkedRoots = new WeakMap();
function __nodeTreeRoot(node) {
  let parent;
  while ((parent = __callHost('parentNode', node)) !== null) node = parent;
  return node;
}
function __nodeIsConnected(node) {
  let root = __nodeTreeRoot(node);
  while (root instanceof ShadowRoot) root = __nodeTreeRoot(__shadowData.get(root).host);
  return root instanceof Document;
}
function __radioGroup(input) {
  if (__inputType(input) !== 'radio') return null;
  const name = __getContentAttribute(input, 'name');
  return name === null || name === ''
    ? null
    : { root: __nodeTreeRoot(input), name, form: __formOwner(input) };
}
function __sameRadioGroup(a, b) {
  return a !== null && b !== null && a.root === b.root && a.name === b.name && a.form === b.form;
}
function __rawInputChecked(input) {
  return __inputCheckedStates.has(input)
    ? __inputCheckedStates.get(input)
    : __getContentAttribute(input, 'checked') !== null;
}
// Detached ordinary fragments do not register groups until they have a form
// owner; ShadowRoot and Document roots register them even without a view.
function __registeredRadioGroup(group) {
  return (
    group !== null &&
    (group.form !== null || group.root instanceof Document || group.root instanceof ShadowRoot)
  );
}
function __uncheckRadioPeers(input, inputs) {
  const group = __radioGroup(input);
  if (group === null) return;
  for (const peer of inputs) {
    if (
      peer !== input &&
      __checkedInitialized.has(peer) &&
      __rawInputChecked(peer) &&
      __sameRadioGroup(group, __radioGroup(peer))
    )
      __inputCheckedStates.set(peer, false);
  }
}
function __ensureCheckedRoot(root) {
  if (!(root instanceof Element || root instanceof Document || root instanceof DocumentFragment))
    return [];
  const cached = __checkedRoots.get(root);
  if (cached?.version === __domVersion) return cached.inputs;
  const inputs = (root instanceof HTMLInputElement ? [root] : []).concat(
    __callHost('querySelectorAll', root, 'input').filter(
      (node) => node instanceof HTMLInputElement,
    ),
  );
  for (const input of inputs) {
    if (__checkedInitialized.has(input)) continue;
    __checkedInitialized.add(input);
    if (__rawInputChecked(input) && __registeredRadioGroup(__radioGroup(input)))
      __uncheckRadioPeers(input, inputs);
  }
  __checkedRoots.set(root, { version: __domVersion, inputs });
  return inputs;
}
function __inputChecked(input) {
  __ensureCheckedRoot(__nodeTreeRoot(input));
  return __rawInputChecked(input);
}
function __setInputChecked(input, value) {
  const inputs = __ensureCheckedRoot(__nodeTreeRoot(input)),
    previous = __rawInputChecked(input);
  __inputCheckedStates.set(input, value);
  if (value && !previous) __uncheckRadioPeers(input, inputs);
}
function __resetInputChecked(input) {
  const inputs = __ensureCheckedRoot(__nodeTreeRoot(input)),
    previous = __rawInputChecked(input);
  __inputCheckedStates.delete(input);
  if (!previous && __rawInputChecked(input)) __uncheckRadioPeers(input, inputs);
}
function __checkedMutationBefore(operation, args) {
  const structural = [
    'appendChild',
    'removeChild',
    'insertBefore',
    'replaceChild',
    'setInnerHTML',
    'setTextContent',
  ].includes(operation);
  let attribute = null;
  if (!structural) {
    if (
      !['setAttribute', 'removeAttribute', 'setAttributeNS', 'removeAttributeNS'].includes(
        operation,
      )
    )
      return null;
    const namespaced = operation.endsWith('NS');
    if (namespaced && args[1]) return null;
    attribute = namespaced ? args[2] : __asciiLowercase(args[1]);
    if (!['checked', 'type', 'name', 'form', 'id'].includes(attribute)) return null;
    if (attribute !== 'id' && !(args[0] instanceof HTMLInputElement)) return null;
    if (
      !namespaced &&
      __attributeRecords(args[0]).find((record) => record.name === attribute)?.namespaceURI != null
    )
      return null;
  }
  const roots = new Set(args.filter((node) => node instanceof Node).map(__nodeTreeRoot));
  // Moving a shadow host can change its controls' explicit form association.
  for (const reference of __shadowRegistrations) {
    const shadow = reference.deref();
    if (shadow && roots.has(__nodeTreeRoot(__shadowData.get(shadow).host))) roots.add(shadow);
  }
  const previous = new Map();
  for (const root of roots)
    for (const input of __ensureCheckedRoot(root))
      previous.set(input, {
        group: __radioGroup(input),
        checked: __rawInputChecked(input),
        dirty: __inputCheckedStates.has(input),
      });
  return () => {
    for (const node of args) if (node instanceof Node) roots.add(__nodeTreeRoot(node));
    for (const root of roots) __ensureCheckedRoot(root);
    const target = args[0];
    if (
      attribute === 'checked' &&
      !__inputCheckedStates.has(target) &&
      !previous.get(target)?.checked &&
      __rawInputChecked(target)
    )
      __uncheckRadioPeers(target, __ensureCheckedRoot(__nodeTreeRoot(target)));
    for (const [input, old] of previous) {
      const group = __radioGroup(input);
      // Preserve insertion order when several checked controls move together.
      // An earlier entrant may have deselected a later entrant already.
      if (old.checked && __registeredRadioGroup(group) && !__sameRadioGroup(old.group, group)) {
        if (old.dirty) __inputCheckedStates.set(input, true);
        else __inputCheckedStates.delete(input);
        __uncheckRadioPeers(input, __ensureCheckedRoot(group.root));
      }
    }
  };
}
