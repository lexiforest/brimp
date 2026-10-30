// Static inventory of browser-facing declarations; never execute the adopted code.
import fs from 'node:fs';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';
const require = createRequire(new URL('../../docs/package.json', import.meta.url));
const ts = require('typescript');
export function inventory(root) {
  const base = path.join(root, 'node-env');
  const files = fs
    .readdirSync(base, { recursive: true })
    .filter((f) => f.endsWith('.js'))
    .sort();
  const entries = new Map(),
    fileAudit = [],
    unresolved = [],
    relationships = [],
    aliases = [];
  const codeText = (n) => n?.getText().replace(/\s+/g, ' ').trim();
  const literalKey = (n) =>
    !n
      ? null
      : ts.isIdentifier(n) || ts.isStringLiteral(n) || ts.isNumericLiteral(n)
        ? n.text
        : ts.isComputedPropertyName(n)
          ? literalKey(n.expression)
          : /^Symbol\.[\w]+$/.test(n.getText())
            ? `[${n.getText()}]`
            : null;
  function add(target, name, kind, node, file, info = {}) {
    if (!target || name == null) return;
    target = target === 'globalThis' ? 'window' : target;
    const id = `${target}.${name}`,
      source = node.getSourceFile();
    const location = `${file}:${source.getLineAndCharacterOfPosition(node.getStart(source)).line + 1}`;
    if (!entries.has(id))
      entries.set(id, {
        id,
        target,
        name,
        legacy: { kinds: [], sources: [], body_patterns: [], examples: [] },
      });
    const row = entries.get(id),
      old = row.legacy;
    for (const [key, value] of Object.entries({ kinds: kind, sources: location, ...info })) {
      if (value == null) continue;
      if (!old[key]) old[key] = [];
      if (!old[key].includes(value)) old[key].push(value);
    }
    return row;
  }
  for (const relative of files) {
    const file = `node-env/${relative}`,
      text = fs.readFileSync(path.join(base, relative), 'utf8');
    const source = ts.createSourceFile(file, text, ts.ScriptTarget.Latest, true, ts.ScriptKind.JS);
    if (source.parseDiagnostics.length) throw Error(`Cannot parse ${file}`);
    const helper =
      relative.startsWith('TOOLS/') ||
      relative.startsWith('BASE_CONFIG/') ||
      /_qxVm\./.test(relative);
    const before = [...entries.values()].reduce((n, e) => n + e.legacy.sources.length, 0);
    const defs = [];
    function collect(n) {
      if (ts.isVariableDeclaration(n) || ts.isParameter(n)) defs.push(n);
      ts.forEachChild(n, collect);
    }
    collect(source);
    function resolve(n, at = n) {
      if (!n || !ts.isIdentifier(n)) return n;
      const candidates = defs.filter(
        (d) => d.initializer && d.name.getText() === n.text && d.getStart() <= at.getStart(),
      );
      // Select the nearest declaration in an enclosing lexical scope.
      for (let scope = at; scope; scope = scope.parent) {
        const found = candidates
          .filter((d) => d.getStart() >= scope.getStart() && d.end <= scope.end)
          .at(-1);
        if (found) return found.initializer;
      }
      return n;
    }
    const globals = new Set();
    function findGlobals(n) {
      if (
        ts.isBinaryExpression(n) &&
        n.operatorToken.kind === ts.SyntaxKind.EqualsToken &&
        ts.isIdentifier(n.left)
      ) {
        const name = n.left.text;
        // Legacy constructors/namespace assignments are intentionally undeclared.
        if (
          !defs.some((d) => d.name.getText() === name) &&
          !['key', 'eventObj', 'runEventLoop'].includes(name)
        )
          globals.add(name);
      }
      if (ts.isFunctionDeclaration(n) && n.parent === source && n.name) globals.add(n.name.text);
      ts.forEachChild(n, findGlobals);
    }
    findGlobals(source);
    const instanceAliases = new Map();
    function findInstances(n) {
      if (ts.isCallExpression(n) && n.expression.getText() === 'Object.setPrototypeOf') {
        const [object, proto] = n.arguments,
          match = proto?.getText().match(/^([A-Z]\w*)\.prototype$/);
        if (
          object &&
          match &&
          !/\.prototype$/.test(object.getText()) &&
          !globals.has(object.getText()) &&
          !/^[A-Z]/.test(object.getText())
        )
          instanceAliases.set(object.getText(), `${match[1]}.instance`);
      }
      ts.forEachChild(n, findInstances);
    }
    findInstances(source);
    function publicTarget(n) {
      if (!n) return null;
      const str = n.getText();
      if (
        /^[\w$]+(?:\.prototype)?$/.test(str) &&
        (globals.has(str.split('.')[0]) || /^[A-Z]/.test(str))
      )
        return str;
      if (['window', 'document', 'location', 'navigator', 'CSS'].includes(str)) return str;
      if (instanceAliases.has(str)) return instanceAliases.get(str);
      return null;
    }
    function ownerOfThis(n) {
      for (let a = n.parent; a; a = a.parent) {
        if (ts.isFunctionDeclaration(a) && a.name && /^[A-Z]/.test(a.name.text))
          return `${a.name.text}.instance`;
        if (
          ts.isFunctionExpression(a) &&
          ts.isBinaryExpression(a.parent) &&
          ts.isIdentifier(a.parent.left) &&
          /^[A-Z]/.test(a.parent.left.text)
        )
          return `${a.parent.left.text}.instance`;
        if (
          (ts.isMethodDeclaration(a) ||
            ts.isGetAccessorDeclaration(a) ||
            ts.isSetAccessorDeclaration(a)) &&
          ts.isObjectLiteralExpression(a.parent) &&
          ts.isBinaryExpression(a.parent.parent)
        )
          return publicTarget(a.parent.parent.left)?.replace(/\.prototype$/, '.instance');
      }
      return null;
    }
    function pattern(body) {
      if (!body) return 'declaration_only';
      if (!body.statements?.length) return 'empty_no_op';
      const s = body.getText();
      if (/let result = undefined;/.test(s) && /return result;/.test(s))
        return 'undefined_return_template';
      if (/new Promise\(function\s*\([^)]*\)\s*\{\s*(?:debugger;?)?\s*\}\)/.test(s))
        return 'never_settling_promise_template';
      return 'custom_body';
    }
    function members(target, obj, origin = 'object_literal') {
      if (!ts.isObjectLiteralExpression(obj)) return;
      for (const p of obj.properties) {
        if (!p.name) continue;
        const name = literalKey(p.name);
        const kind = ts.isGetAccessorDeclaration(p)
          ? 'get'
          : ts.isSetAccessorDeclaration(p)
            ? 'set'
            : ts.isMethodDeclaration(p) || (p.initializer && ts.isFunctionExpression(p.initializer))
              ? 'method'
              : 'data';
        const body = p.body ?? p.initializer?.body;
        add(target, name, kind, p, file, {
          body_patterns: body ? pattern(body) : null,
          declaration_forms: origin,
          examples: ts.isPropertyAssignment(p) && !body ? codeText(p.initializer) : null,
        });
      }
    }
    function mapMembers(target, expr, kind, at, mapKind) {
      const obj = resolve(expr, at);
      if (!obj || !ts.isObjectLiteralExpression(obj)) {
        unresolved.push({
          source: `${file}:${source.getLineAndCharacterOfPosition(at.getStart()).line + 1}`,
          expression: codeText(expr),
          reason: 'unresolved_declaration_map',
        });
        return;
      }
      for (const p of obj.properties)
        if (p.name) {
          const name = literalKey(p.name),
            val = p.initializer;
          if (mapKind === 'accessor_map' && val && ts.isArrayLiteralExpression(val)) {
            if (val.elements[1]?.kind === ts.SyntaxKind.TrueKeyword)
              add(target, name, 'get', p, file, {
                declaration_forms: mapKind,
                examples: codeText(val.elements[0]),
              });
            if (val.elements[2]?.kind === ts.SyntaxKind.TrueKeyword)
              add(target, name, 'set', p, file, { declaration_forms: mapKind });
          } else
            add(target, name, kind, p, file, {
              declaration_forms: mapKind,
              examples: codeText(val),
            });
        }
    }
    function visit(n) {
      if (
        relative === 'BASE_CONFIG/Del_attribute.js' &&
        ts.isVariableDeclaration(n) &&
        n.name.getText() === 'del_attribute' &&
        ts.isArrayLiteralExpression(n.initializer)
      ) {
        for (const item of n.initializer.elements)
          add('window', literalKey(item), 'removed', item, file, {
            declaration_forms: 'explicit_global_removal',
          });
      }
      if (
        ts.isCallExpression(n) &&
        n.expression.getText() === 'ReadCode' &&
        ts.isStringLiteral(n.arguments[0])
      ) {
        const candidate = path.join(
          path.dirname(path.join(base, relative)),
          n.arguments[1]?.text ?? '',
          n.arguments[0].text,
        );
        if (!fs.existsSync(candidate))
          unresolved.push({
            source: file,
            expression: path.relative(root, candidate),
            reason: 'missing_loader_dependency',
          });
      }
      if (helper) {
        ts.forEachChild(n, visit);
        return;
      }
      if (ts.isFunctionDeclaration(n) && n.parent === source && n.name)
        add('window', n.name.text, /^[A-Z]/.test(n.name.text) ? 'constructor' : 'method', n, file, {
          body_patterns: pattern(n.body),
        });
      if (ts.isBinaryExpression(n) && n.operatorToken.kind === ts.SyntaxKind.EqualsToken) {
        const left = n.left,
          right = n.right,
          target = publicTarget(left);
        if (ts.isIdentifier(left) && globals.has(left.text)) {
          add(
            'window',
            left.text,
            ts.isFunctionExpression(right)
              ? /^[A-Z]/.test(left.text)
                ? 'constructor'
                : 'method'
              : 'data',
            n,
            file,
            {
              body_patterns: right.body ? pattern(right.body) : null,
              examples: right.body ? null : codeText(right).slice(0, 180),
            },
          );
          if (ts.isObjectLiteralExpression(right)) members(left.text, right);
        } else if (target && /\.prototype$/.test(target)) {
          add(target.replace(/\.prototype$/, ''), 'prototype', 'data', n, file, {
            declaration_forms: 'prototype_assignment',
          });
          if (ts.isObjectLiteralExpression(right)) members(target, right);
          else if (/\.prototype$/.test(right.getText()))
            aliases.push({ target, source_target: right.getText(), source: file });
        } else if (ts.isPropertyAccessExpression(left) || ts.isElementAccessExpression(left)) {
          const owner =
            left.expression.kind === ts.SyntaxKind.ThisKeyword
              ? ownerOfThis(n)
              : publicTarget(left.expression);
          const dynamic =
            ts.isElementAccessExpression(left) &&
            !ts.isStringLiteral(left.argumentExpression) &&
            !ts.isNumericLiteral(left.argumentExpression) &&
            !/^Symbol\./.test(left.argumentExpression.getText());
          const key = ts.isPropertyAccessExpression(left)
            ? left.name.text
            : dynamic
              ? '[name]'
              : literalKey(left.argumentExpression);
          if (owner)
            add(
              owner,
              key,
              dynamic
                ? 'dynamic'
                : ts.isFunctionExpression(right) || ts.isArrowFunction(right)
                  ? 'method'
                  : 'data',
              n,
              file,
              {
                declaration_forms: dynamic ? 'dynamic_named_assignment' : 'assignment',
                body_patterns: right.body ? pattern(right.body) : null,
                examples: right.body ? null : codeText(right).slice(0, 180),
              },
            );
        }
      }
      if (ts.isVariableDeclaration(n) && n.initializer) {
        const name = n.name.getText();
        if (name === 'get_set_attribute_map')
          mapMembers('window', n.initializer, 'data', n, 'accessor_map');
        if (name === 'value_attribute_map_string')
          mapMembers('window', n.initializer, 'data', n, 'global_value_map');
        if (name === 'timer' && relative === 'BOM/timer.js')
          members('window', n.initializer, 'timer_export');
        if (name === '$cssstyle_attribute' && ts.isArrayLiteralExpression(n.initializer))
          for (const item of n.initializer.elements) {
            add('CSSStyleDeclaration.instance', literalKey(item), 'data', item, file, {
              declaration_forms: 'css_property_list',
              examples: '""',
            });
          }
      }
      if (ts.isCallExpression(n)) {
        const call = n.expression.getText(),
          args = n.arguments;
        if (call === 'lwVm.safe_Objattribute') {
          const target =
            args[0].getText() === 'location' || args[4]?.kind === ts.SyntaxKind.TrueKeyword
              ? args[0].getText()
              : `${args[0].getText()}.prototype`;
          for (const [i, kind] of [
            [1, 'get'],
            [2, 'set'],
            [3, 'method'],
          ]) {
            const list = resolve(args[i], n);
            if (list && ts.isArrayLiteralExpression(list))
              for (const item of list.elements)
                add(target, literalKey(item), kind, item, file, {
                  declaration_forms: 'safe_member_list',
                });
          }
        }
        if (call === 'lwVm.add_capitalLetter_attribute')
          for (const target of [args[0].getText(), `${args[0].getText()}.prototype`])
            mapMembers(target, args[1], 'constant', n, 'constant_helper');
        if (call === 'lwVm.safeDescriptor_addConstructor')
          add(`${args[0].getText()}.prototype`, 'constructor', 'data', n, file, {
            declaration_forms: 'constructor_helper',
          });
        if (call === 'lwVm.rename' && publicTarget(args[0]))
          add(args[0].getText(), '[Symbol.toStringTag]', 'data', n, file, {
            declaration_forms: 'tag_helper',
            examples: codeText(args[1]),
          });
        if (call === 'lwVm.set_iterator')
          add(`${args[0].getText()}.prototype`, '[Symbol.iterator]', 'method', n, file, {
            declaration_forms: 'iterator_helper',
          });
        if (call === 'lwVm.protectAddIsTrusted')
          add(ownerOfThis(n), 'isTrusted', 'get', n, file, {
            declaration_forms: 'trusted_event_helper',
          });
        if (call === 'Object.setPrototypeOf' && publicTarget(args[0]) && publicTarget(args[1]))
          relationships.push({
            target: args[0].getText(),
            parent: args[1].getText(),
            source: file,
          });
        if (call === 'Object.defineProperty' && args[0]) {
          let target =
            args[0].kind === ts.SyntaxKind.ThisKeyword ? ownerOfThis(n) : publicTarget(args[0]);
          if (relative === 'BOM/Plugin/Plugin.js' && args[0].getText() === 'plugin')
            target = 'Plugin.instance';
          if (args[0].getText() === 'lwVm.memory.pluginArray.instantiate')
            target = 'PluginArray.instance';
          if (args[0].getText() === 'lwVm.memory.mimeTypeArray.instantiate')
            target = 'MimeTypeArray.instance';
          if (target) {
            const key = literalKey(args[1]);
            const descriptor = resolve(args[2], n);
            if (key !== null && !(ts.isIdentifier(args[1]) && !args[1].text.startsWith('Symbol'))) {
              const kinds =
                descriptor && ts.isObjectLiteralExpression(descriptor)
                  ? descriptor.properties
                      .map((p) => p.name && literalKey(p.name))
                      .filter((k) => ['get', 'set'].includes(k))
                  : [];
              for (const kind of kinds.length ? kinds : ['data'])
                add(target, key, kind, n, file, { declaration_forms: 'defineProperty' });
            } else if (
              /^WebGL|XMLHttpRequest|HTMLMediaElement/.test(target) &&
              args[1]?.getText() === 'key'
            )
              mapMembers(
                target,
                ts.factory.createIdentifier('$attributes'),
                'constant',
                n,
                'constant_loop',
              );
            else if (relative === 'DOM/document_.js' && target === 'document') {
              mapMembers(
                target,
                ts.factory.createIdentifier('$attribute_map'),
                'get',
                n,
                'document_instance_map',
              );
              mapMembers(
                target,
                ts.factory.createIdentifier('$attribute_map'),
                'set',
                n,
                'document_instance_map',
              );
            } else if (target.endsWith('.instance') && /Plugin|MimeType/.test(target)) {
              const dynamicKey = /index|\.length$/.test(args[1].getText()) ? '[index]' : '[name]';
              add(target, dynamicKey, 'dynamic', n, file, {
                declaration_forms: 'dynamic_named_or_indexed_property',
                examples: codeText(args[1]),
              });
            } else if (
              !(target === 'window' && ['key', 'timer_obj.name'].includes(args[1]?.getText())) &&
              target !== 'location'
            )
              unresolved.push({
                source: file,
                expression: n.getText().slice(0, 220),
                reason: 'dynamic_property_definition',
              });
          }
        }
        if (
          call === 'Object.defineProperties' &&
          publicTarget(args[0]) &&
          ts.isObjectLiteralExpression(args[1])
        )
          for (const prop of args[1].properties) {
            if (!prop.name) continue;
            const target = args[0].getText(),
              name = literalKey(prop.name);
            add(target, name, 'data', prop, file, {
              declaration_forms: 'defineProperties',
              examples: codeText(prop.initializer),
            });
            const descriptor = prop.initializer;
            const val = descriptor?.properties?.find(
              (p) => p.name?.getText() === 'value',
            )?.initializer;
            if (name && val && ts.isObjectLiteralExpression(val))
              members(`${target}.${name}`, val, 'nested_descriptor_value');
          }
      }
      ts.forEachChild(n, visit);
    }
    visit(source);
    const count = [...entries.values()].reduce((n, e) => n + e.legacy.sources.length, 0) - before;
    fileAudit.push({
      path: file,
      sha256: createHash('sha256').update(text).digest('hex'),
      role:
        relative === 'BASE_CONFIG/Del_attribute.js'
          ? 'explicit_global_removals'
          : helper
            ? 'loader_or_harness_helper'
            : count
              ? 'browser_surface'
              : 'internal_implementation',
      declaration_locations: count,
    });
  }
  const rows = [...entries.values()].sort((a, b) => a.id.localeCompare(b.id, 'en'));
  for (const row of rows)
    for (const key of Object.keys(row.legacy)) if (!row.legacy[key].length) delete row.legacy[key];
  return { rows, files: fileAudit, unresolved, relationships, aliases };
}
