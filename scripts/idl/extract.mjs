// Extract the interface surface from WebKit's IDL sources.
//
// This is the WebKit analogue of pyv8c's `tools/extract_idl.py`, which reads the
// member tables Chromium's binding generator emits. WebKit ships the IDL itself,
// so the shape comes from `Source/WebCore/**/*.idl` at the pinned commit and no
// generated C++ has to be parsed.
//
// The document records *shape*: parent chain, exposure, member names, kinds,
// arity, readonly, constants and aliases. Which of those members a given browser
// installs is a profile question, answered by `scripts/idl/capture.mjs`.
import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
const webidl2 = require('webidl2');

const DOCUMENT_VERSION = 1;
// Test-only IDL describes harness fixtures (TestEnabledBySetting, TestGlobalObject)
// rather than a browser surface.
const EXCLUDED = [path.join('bindings', 'scripts', 'test')];

// A condition the platform headers do not mention is not a Safari feature, but
// including its branch costs nothing: exposure is decided by the captured profile
// and a member that Safari lacks is simply never installed.
const UNKNOWN_FLAG = 1;
const PLATFORMS = {
  MAC: 1,
  COCOA: 1,
  IOS_FAMILY: 0,
  WATCHOS: 0,
  VISION: 0,
  WIN: 0,
  GTK: 0,
  WPE: 0,
};

function readFeatureFlags(webkitRoot) {
  const flags = new Map();
  for (const file of ['PlatformEnable.h', 'PlatformEnableCocoa.h', 'PlatformHave.h']) {
    let text;
    try {
      text = fs.readFileSync(path.join(webkitRoot, 'Source', 'WTF', 'wtf', file), 'utf8');
    } catch {
      continue;
    }
    for (const [, name, value] of text.matchAll(/^#define\s+([A-Z][A-Z0-9_]+)\s+([0-9]+)\s*$/gm))
      flags.set(name, Number(value));
  }
  return flags;
}

// The build's own headers gate a handful of IDL files with the C preprocessor.
function evaluate(expression, flags) {
  const tokens = expression.match(/defined|[A-Za-z_][A-Za-z0-9_]*|[0-9]+|&&|\|\||[!()]/g) ?? [];
  let index = 0;
  const call = () => {
    const args = [];
    if (tokens[index] !== '(') return args;
    index += 1;
    while (index < tokens.length && tokens[index] !== ')') {
      const token = tokens[index++];
      if (token !== ',') args.push(token);
    }
    index += 1;
    return args;
  };
  const primary = () => {
    const token = tokens[index++];
    if (token === undefined) return 0;
    if (token === '!') return primary() ? 0 : 1;
    if (token === '(') {
      const value = disjunction();
      index += 1;
      return value;
    }
    if (token === 'defined') {
      if (tokens[index] === '(') {
        index += 1;
        const name = tokens[index++];
        index += 1;
        return flags.has(name) ? 1 : UNKNOWN_FLAG;
      }
      const name = tokens[index++];
      return flags.has(name) ? 1 : UNKNOWN_FLAG;
    }
    if (['PLATFORM', 'USE', 'HAVE', 'CPU', 'OS'].includes(token)) {
      const [name] = call();
      if (token === 'PLATFORM') return PLATFORMS[name] ?? 0;
      return flags.get(`${token}_${name}`) ?? flags.get(name) ?? 0;
    }
    if (tokens[index] === '(') {
      // A function-like macro this script has no table for: assume it is on.
      call();
      return UNKNOWN_FLAG;
    }
    if (/^[0-9]+$/.test(token)) return Number(token);
    return flags.get(token) ?? UNKNOWN_FLAG;
  };
  const conjunction = () => {
    let value = primary();
    while (tokens[index] === '&&') {
      index += 1;
      const next = primary();
      value = value && next ? 1 : 0;
    }
    return value;
  };
  const disjunction = () => {
    let value = conjunction();
    while (tokens[index] === '||') {
      index += 1;
      const next = conjunction();
      value = value || next ? 1 : 0;
    }
    return value;
  };
  return disjunction() !== 0;
}

function preprocess(text, flags) {
  const lines = [];
  const stack = [];
  const active = () => stack.every((frame) => frame.active);
  for (const line of text.split('\n')) {
    const directive = line.match(/^\s*#\s*(\w+)\s*(.*)$/);
    if (!directive) {
      if (active()) lines.push(line);
      continue;
    }
    const [, name, rest] = directive;
    if (name === 'if') {
      const value = evaluate(rest, flags);
      stack.push({ active: value, taken: value });
    } else if (name === 'ifdef') {
      const value = evaluate(`defined(${rest.trim()})`, flags);
      stack.push({ active: value, taken: value });
    } else if (name === 'ifndef') {
      const value = !evaluate(`defined(${rest.trim()})`, flags);
      stack.push({ active: value, taken: value });
    } else if (name === 'elif') {
      const frame = stack.at(-1);
      if (!frame) continue;
      const value = !frame.taken && evaluate(rest, flags);
      frame.active = value;
      frame.taken ||= value;
    } else if (name === 'else') {
      const frame = stack.at(-1);
      if (!frame) continue;
      frame.active = !frame.taken;
      frame.taken = true;
    } else if (name === 'endif') {
      stack.pop();
    }
  }
  return lines.join('\n');
}

// WebKit's IDL predates a few WebIDL grammar rules: extended attribute lists end
// with a trailing comma, attribute values spell reserved tokens and `&`/`|`
// combinations unquoted, keywords appear as declaration names (`object`, `record`,
// `readonly`, `toString`), and `any` appears in union types. Rewrite those
// spellings so a strict parser reads the same declarations.
const KEYWORDS =
  'FrozenArray|ObservableArray|Promise|bigint|boolean|byte|double|float|long|octet|record|sequence|short|undefined|unsigned|void|any|object|readonly|required';
const NAME_POSITION = new RegExp(
  `(?<=[\\w>\\]] )(?<!unrestricted )(?<!unsigned )\\b(${KEYWORDS})\\b(?=\\s*[,);=])`,
  'g',
);

function normalize(text) {
  return text
    .replace(/=\s*(\[\]|\{\})/g, '= "$1"')
    .replace(/,\s*\]/g, ']')
    .replace(/\[[^[\]]*\]/g, (list) =>
      list
        .replace(/=\s*(null|true|false|any|\[\]|\{\})\s*(?=,|\])/g, '= "$1"')
        .replace(/=\s*([A-Za-z_][\w.]*(?:\s*[&|]\s*[A-Za-z_][\w.]*)+)/g, '= "$1"'),
    )
    .replace(NAME_POSITION, '$1_')
    .replace(/\btoString\b(?=\s*[;(])/g, 'toString_')
    .replace(/\bor\s+any\b/g, 'or DOMString');
}

function attribute(arguments_, name) {
  return arguments_.find((argument) => argument.name === name);
}

function attributeValue(member, name) {
  const found = attribute(member.extAttrs, name);
  const rhs = found?.rhs;
  if (rhs === null || rhs === undefined) return undefined;
  if (typeof rhs.value === 'string') return rhs.value;
  if (rhs.value && typeof rhs.value === 'object' && 'value' in rhs.value) return rhs.value.value;
  return rhs.value;
}

function hasAttribute(arguments_, name) {
  const extAttrs = Array.isArray(arguments_) ? arguments_ : arguments_.extAttrs;
  return attribute(extAttrs, name) !== undefined;
}

// An extended attribute's right-hand side is an identifier, an identifier list or
// a token list; only the first two name things.
function rhsNames(member, name) {
  const found = attribute(member.extAttrs, name);
  const rhs = found?.rhs;
  if (!rhs) return [];
  if (rhs.type === 'identifier' || typeof rhs.value === 'string') return [String(rhs.value)];
  if (!Array.isArray(rhs.value)) return [];
  return rhs.value.map((value) => (typeof value === 'string' ? value : String(value.value)));
}

function exposure(definition) {
  const names = rhsNames(definition, 'Exposed');
  return names.length ? names : ['Window'];
}

function requiredArguments(operation) {
  if (operation.special === 'stringifier' && !operation.name) return null;
  let count = 0;
  for (const argument of operation.arguments) {
    if (argument.optional || argument.default || argument.variadic) break;
    count += 1;
  }
  return count;
}

function constantValue(member) {
  const value = member.value?.value;
  if (typeof value === 'number' || typeof value === 'boolean' || typeof value === 'string')
    return value;
  if (value && typeof value === 'object' && 'value' in value) return value.value;
  return null;
}

const REWRITTEN = {
  toString_: 'toString',
  readonly_: 'readonly',
  object_: 'object',
  record_: 'record',
  any_: 'any',
};
const restore = (name) => REWRITTEN[name] ?? name;

// WebKit hides features behind settings (`[EnabledBySetting=…]`) and platform
// macros (`[Conditional=…]`); a shipped release can leave them off.
const GATES = [
  'EnabledBySetting',
  'EnabledForContext',
  'EnabledForWorld',
  'Conditional',
  'DisabledByQuirk',
];
const isGated = (extAttrs) => GATES.some((gate) => attribute(extAttrs, gate) !== undefined);

function memberRecord(location, member) {
  if (member.type === 'constructor')
    return { name: 'constructor', kind: 'operation', length: requiredArguments(member), location };
  if (member.type === 'attribute') {
    const record = { name: restore(member.name), kind: 'attribute' };
    const writable =
      !member.readonly ||
      hasAttribute(member, 'PutForwards') ||
      hasAttribute(member, 'Replaceable') ||
      hasAttribute(member, 'LegacyLenientSetter');
    if (writable) record.setter = true;
    if (hasAttribute(member, 'PutForwards'))
      record.put_forwards = attributeValue(member, 'PutForwards');
    if (hasAttribute(member, 'SameObject')) record.same_object = true;
    if (hasAttribute(member, 'Replaceable')) record.replaceable = true;
    if (member.static) record.static = true;
    if (isGated(member.extAttrs)) record.gated = true;
    record.location = location;
    return record;
  }
  if (member.type === 'operation') {
    const length = requiredArguments(member);
    if (length === null) return { name: 'toString', kind: 'operation', length: 0, location };
    const record = { name: restore(member.name), kind: 'operation', length, location };
    if (member.static) record.static = true;
    if (member.special === 'stringifier') record.stringifier = true;
    if (isGated(member.extAttrs)) record.gated = true;
    return record;
  }
  if (member.type === 'const') {
    return { name: restore(member.name), kind: 'const', value: constantValue(member), location };
  }
  return null;
}

// `iterable`, `maplike` and `setlike` install their own member set.
// The member set an `iterable`, `maplike` or `setlike` declaration installs,
// with the arity each operation has.
function iterationMembers(member) {
  if (member.type === 'iterable')
    return [
      { name: 'entries', length: 0 },
      { name: 'keys', length: 0 },
      { name: 'values', length: 0 },
      { name: 'forEach', length: 1 },
      { name: 'Symbol.iterator', length: 0 },
    ];
  if (member.type === 'async_iterable')
    return [
      { name: 'values', length: 0 },
      { name: 'Symbol.asyncIterator', length: 0 },
    ];
  if (member.type === 'maplike')
    return [
      { name: 'size', kind: 'attribute' },
      { name: 'get', length: 1 },
      { name: 'has', length: 1 },
      { name: 'delete', length: 1 },
      { name: 'set', length: 2 },
      { name: 'clear', length: 0 },
      { name: 'entries', length: 0 },
      { name: 'keys', length: 0 },
      { name: 'values', length: 0 },
      { name: 'forEach', length: 1 },
      { name: 'Symbol.iterator', length: 0 },
    ];
  if (member.type === 'setlike')
    return [
      { name: 'size', kind: 'attribute' },
      { name: 'has', length: 1 },
      { name: 'add', length: 1 },
      { name: 'delete', length: 1 },
      { name: 'clear', length: 0 },
      { name: 'entries', length: 0 },
      { name: 'keys', length: 0 },
      { name: 'values', length: 0 },
      { name: 'forEach', length: 1 },
      { name: 'Symbol.iterator', length: 0 },
    ];
  return [];
}

function sortUnique(records) {
  const seen = new Map();
  for (const record of records) if (!seen.has(record.name)) seen.set(record.name, record);
  return [...seen.values()].sort((a, b) => (a.name < b.name ? -1 : a.name > b.name ? 1 : 0));
}

function collect(files, flags) {
  const interfaces = new Map();
  const mixinMembers = new Map();
  const includes = [];
  const report = { files: files.length, parsed: 0, skipped: [], interfaces_parsed: 0, members: 0 };

  // Two passes: the public name of an interface (`[InterfaceName]`) is stated on
  // its main definition only, and `partial interface` definitions add members to
  // the interface they name in IDL, which is not necessarily the public name.
  const definitions = [];
  for (const file of files) {
    let ast;
    try {
      ast = webidl2.parse(normalize(preprocess(fs.readFileSync(file, 'utf8'), flags)));
    } catch (error) {
      report.skipped.push({
        file,
        reason: String(error.message).split('\n').slice(0, 3).join(' '),
      });
      continue;
    }
    report.parsed += 1;
    definitions.push(...ast.map((definition) => ({ definition, file })));
  }

  const publicNames = new Map();
  for (const { definition } of definitions)
    if (
      (definition.type === 'interface' && !definition.partial) ||
      (definition.type === 'namespace' && !definition.partial)
    )
      publicNames.set(definition.name, rhsNames(definition, 'InterfaceName')[0] ?? definition.name);
  const publicName = (idlName) => publicNames.get(idlName) ?? idlName;

  const entryFor = (name) => {
    if (!interfaces.has(name))
      interfaces.set(name, {
        parent: null,
        exposed_in: ['Window'],
        members: { prototype: [], interface: [], instance: [] },
        constants: [],
      });
    return interfaces.get(name);
  };

  const addMembers = (entry, members, global) => {
    for (const member of members) {
      for (const generated of iterationMembers(member)) {
        entry.members.prototype.push({
          name: generated.name,
          kind: generated.kind ?? 'operation',
          length: generated.length ?? 0,
          generated: true,
        });
        report.members += 1;
      }
      const record = memberRecord(member.static ? 'interface' : 'prototype', member);
      if (!record) continue;
      report.members += 1;
      if (record.kind === 'const') entry.constants.push({ name: record.name, value: record.value });
      // `constructor` is a key every object inherits; the document names it
      // `construction` so a lookup cannot fall back to `Object`.
      else if (record.name === 'constructor' && record.kind === 'operation')
        entry.construction = { allow_new: true, allow_call: false, length: record.length };
      else if (record.location === 'interface') entry.members.interface.push(record);
      // A `[Global]` interface's attributes are own properties of the global
      // object (its operations stay on the prototype); a namespace is a plain
      // object, so everything it declares is an own property.
      else if (global === 'all' || (global && record.kind === 'attribute'))
        entry.members.instance.push(record);
      else entry.members.prototype.push(record);
    }
  };

  for (const { definition } of definitions) {
    if (definition.type === 'interface') {
      const entry = entryFor(publicName(definition.name));
      if (!definition.partial) {
        report.interfaces_parsed += 1;
        // `inheritance` is the parent's name, not an object.
        if (definition.inheritance)
          entry.parent =
            typeof definition.inheritance === 'string'
              ? definition.inheritance
              : (definition.inheritance.name ?? null);
        entry.exposed_in = exposure(definition);
        const aliases = rhsNames(definition, 'LegacyWindowAlias');
        if (aliases.length) entry.aliases = aliases;
        const factory = rhsNames(definition, 'LegacyFactoryFunction');
        if (factory.length) entry.factory = factory;
        if (hasAttribute(definition.extAttrs, 'LegacyNoInterfaceObject'))
          entry.no_interface_object = true;
        if (hasAttribute(definition.extAttrs, 'SecureContext')) entry.secure_context = true;
        if (isGated(definition.extAttrs)) entry.gated = true;
        if (hasAttribute(definition.extAttrs, 'Global')) entry.global = true;
        entry.declared = true;
      }
      // A `[Global]` interface is the global object itself, so everything it
      // declares is an own property of `window`.
      addMembers(entry, definition.members, entry.global === true ? 'all' : false);
    } else if (definition.type === 'interface mixin') {
      if (!mixinMembers.has(definition.name)) mixinMembers.set(definition.name, []);
      mixinMembers.get(definition.name).push(...definition.members);
    } else if (definition.type === 'namespace') {
      const entry = entryFor(publicName(definition.name));
      entry.namespace = true;
      entry.declared = true;
      entry.exposed_in = exposure(definition);
      addMembers(entry, definition.members, 'all');
    } else if (definition.type === 'callback interface') {
      // A callback interface is a bucket of constants in a page realm; its
      // operations are for the implementation, not for script.
      const entry = entryFor(publicName(definition.name));
      entry.callback_interface = true;
      entry.declared = true;
      entry.exposed_in = exposure(definition);
      for (const member of definition.members) {
        const record = memberRecord('interface', member);
        if (record?.kind === 'const')
          entry.constants.push({ name: record.name, value: record.value });
      }
    } else if (definition.type === 'includes') {
      includes.push({ target: publicName(definition.target), mixin: definition.includes });
    }
  }

  for (const { target, mixin } of includes) {
    const entry = interfaces.get(target);
    const members = mixinMembers.get(mixin);
    if (!entry || !members) continue;
    addMembers(entry, members, entry.global === true);
  }

  for (const [name, entry] of interfaces) {
    for (const location of ['prototype', 'interface', 'instance'])
      entry.members[location] = sortUnique(entry.members[location]);
    entry.constants = sortUnique(entry.constants);
    // An interface may declare no members at all (`Comment`, `XMLDocument`) and
    // still be a global constructor.
    if (!entry.declared) interfaces.delete(name);
  }
  return { interfaces, report };
}

function gitCommit(root) {
  try {
    return fs
      .readFileSync(path.join(root, '.git', 'HEAD'), 'utf8')
      .trim()
      .slice(0, 12);
  } catch {
    return null;
  }
}

function main() {
  const args = process.argv.slice(2);
  const option = (name, fallback) => {
    const index = args.indexOf(`--${name}`);
    return index === -1 ? fallback : args[index + 1];
  };
  const root = path.resolve(
    option('webkit-root', path.join(import.meta.dirname, '..', '..', '..', 'WebKit')),
  );
  const output = path.resolve(
    option(
      'output',
      path.join(import.meta.dirname, '..', '..', 'crates', 'runtime', 'data', 'webkit-idl.json'),
    ),
  );
  const webcore = path.join(root, 'Source', 'WebCore');
  const walk = (directory, found = []) => {
    for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
      const full = path.join(directory, entry.name);
      if (entry.isDirectory()) walk(full, found);
      else if (entry.name.endsWith('.idl')) found.push(full);
    }
    return found;
  };
  const files = walk(webcore)
    .filter((file) => !EXCLUDED.some((excluded) => file.includes(excluded)))
    .sort();
  const flags = readFeatureFlags(root);
  const { interfaces, report } = collect(files, flags);
  const document = {
    idl: {
      version: DOCUMENT_VERSION,
      source: 'scripts/idl/extract.mjs',
      webkit_commit: gitCommit(root),
      report: {
        ...report,
        interfaces: interfaces.size,
        interfaces_with_members: [...interfaces.values()].filter(
          (entry) => entry.members.prototype.length,
        ).length,
        window_exposed: [...interfaces.values()].filter((entry) =>
          entry.exposed_in.includes('Window'),
        ).length,
      },
    },
    interfaces: Object.fromEntries(
      [...interfaces].sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0)),
    ),
  };
  fs.mkdirSync(path.dirname(output), { recursive: true });
  fs.writeFileSync(output, `${JSON.stringify(document, null, 1)}\n`);
  const bytes = fs.statSync(output).size;
  console.log(
    `${interfaces.size} interfaces, ${report.members} members from ${report.parsed}/${report.files} files ` +
      `(${report.skipped.length} skipped) -> ${path.relative(process.cwd(), output)} (${(bytes / 1024).toFixed(0)} KiB)`,
  );
  if (report.skipped.length) console.log(JSON.stringify(report.skipped, null, 1));
}

if (process.argv[1] === new URL(import.meta.url).pathname) main();

export { evaluate, preprocess, normalize, readFeatureFlags };
