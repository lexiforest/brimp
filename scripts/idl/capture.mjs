// Capture the Window surface of the Safari instance this machine runs.
//
// The reference has to come from a browser a person is using, not from an
// automated one: `safaridriver` sets `navigator.webdriver` and installs its own
// automation hooks, which changes what a page sees. This script asks Safari for
// the JavaScript result of `do JavaScript … in front document`, so open Safari on
// an ordinary page before running it:
//
//     open -a Safari https://example.com
//     node scripts/idl/capture.mjs
//
// The result records the names `Object.getOwnPropertyNames(window)` reports and,
// for each one, the descriptor shape of the object behind it. `scripts/idl/extract.mjs`
// supplies the shape those names should have; this file says which ones Safari
// exposes and in what order.
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { execFileSync } from 'node:child_process';
import { PROBE as probeSource } from './probe.mjs';

// Two artifacts, written together so they cannot disagree: the runtime embeds
// the installation order and exposure, and the parity gate reads the full
// descriptor record of every object on the page.
const ROOT = path.join(import.meta.dirname, '..', '..');
const OUTPUT = path.resolve(
  process.argv[2] ?? path.join(ROOT, 'tests', 'reference', 'safari-surface.json'),
);
const RUNTIME_OUTPUT = path.resolve(
  process.argv[3] ?? path.join(ROOT, 'crates', 'runtime', 'data', 'safari-window.json'),
);

const PROBE = probeSource;

function osascript(expression) {
  return execFileSync('osascript', ['-e', expression], { encoding: 'utf8' }).trim();
}

function main() {
  const script = path.join(fs.mkdtempSync(path.join(os.tmpdir(), 'brimp-idl-')), 'probe.js');
  fs.writeFileSync(script, PROBE);
  const captured = osascript(
    `tell application "Safari" to do JavaScript (read file (POSIX file "${script}" as text) as «class utf8») in front document`,
  );
  fs.rmSync(path.dirname(script), { recursive: true, force: true });
  let report;
  try {
    report = JSON.parse(captured);
  } catch {
    throw new Error(`Safari returned something other than JSON: ${captured.slice(0, 200)}`);
  }
  if (report.webdriver)
    throw new Error(
      'This Safari is under automation (navigator.webdriver); capture from a normal window',
    );
  if (report.surface.names.length < 500)
    throw new Error(
      `Only ${report.surface.names.length} window names: open a normal https page in Safari first (got ${report.url})`,
    );
  const safari = osascript('tell application "Safari" to get version');
  const document = {
    source: 'scripts/idl/capture.mjs',
    safari_version: safari,
    captured_at: new Date().toISOString(),
    url: report.url,
    user_agent: report.user_agent,
    ...report.surface,
  };
  fs.mkdirSync(path.dirname(OUTPUT), { recursive: true });
  fs.writeFileSync(OUTPUT, `${JSON.stringify(document, null, 1)}\n`);
  // The runtime embeds the installation order, the descriptors of the window's
  // own properties, and the shape of every name WebKit's IDL does not describe
  // (`GestureEvent`, `SVGPathSeg`, Safari's own globals). Everything else the
  // IDL document already states.
  const idlPath = path.join(ROOT, 'crates', 'runtime', 'data', 'webkit-idl.json');
  const idl = fs.existsSync(idlPath) ? JSON.parse(fs.readFileSync(idlPath, 'utf8')).interfaces : {};
  const described = new Set(Object.keys(idl));
  // Engine built-ins (`Object`, `Math`, `Infinity`, …) are not WebIDL and every
  // JavaScript engine already provides them; the ones this script's own realm
  // shares are skipped.
  const extra = {};
  for (const [name, entry] of Object.entries(document.entries))
    if (!described.has(name) && !(name in globalThis)) extra[name] = entry;
  // What this Safari leaves out of the interfaces WebKit declares: the members
  // behind a setting the release ships off. Everything else the installer reads
  // from the IDL, so only the difference travels with the runtime.
  const omitted = {};
  for (const [name, entry] of Object.entries(document.entries)) {
    const declaration = idl[name];
    if (declaration === undefined) continue;
    // `Object.getOwnPropertyNames` never reports symbol members, so a symbol the
    // document generates is not evidence that Safari omits it.
    const declared = (location) =>
      new Set(
        declaration.members[location]
          .map((member) => member.name)
          .filter((member) => !member.startsWith('Symbol.')),
      );
    // Absence from the prototype only proves omission for members a setting
    // gates: WebKit installs plain ones such as `document.location` on each
    // instance, where this probe cannot see them.
    const prototype = new Set(
      declaration.members.prototype
        .filter((member) => member.gated === true)
        .map((member) => member.name),
    );
    const interfaceObject = new Set([
      ...declared('interface'),
      ...declaration.constants.map((constant) => constant.name),
    ]);
    const present = (set) => new Set(Object.keys(set ?? {}));
    const left = (want, have) => [...want].filter((member) => !have.has(member)).sort();
    const prototypeOmitted = left(prototype, present(entry.proto));
    const interfaceOmitted = left(interfaceObject, present(entry.own));
    if (prototypeOmitted.length || interfaceOmitted.length)
      omitted[name] = {
        ...(prototypeOmitted.length ? { prototype: prototypeOmitted } : {}),
        ...(interfaceOmitted.length ? { interface: interfaceOmitted } : {}),
      };
    // A constant WebKit declares but Safari lacks is removed in both places.
    const constantsLeft = declaration.constants.filter(
      (constant) => !(constant.name in (entry.own ?? {})),
    );
    if (constantsLeft.length)
      omitted[name].constants = constantsLeft.map((constant) => constant.name);
  }
  const runtime = {
    source: document.source,
    safari_version: document.safari_version,
    captured_at: document.captured_at,
    url: document.url,
    user_agent: document.user_agent,
    names: document.names,
    window: document.window,
    extra,
    omitted,
  };
  fs.mkdirSync(path.dirname(RUNTIME_OUTPUT), { recursive: true });
  fs.writeFileSync(RUNTIME_OUTPUT, `${JSON.stringify(runtime)}\n`);
  console.log(
    `${document.names.length} window names, ${Object.keys(document.entries).length} objects ` +
      `from Safari ${safari} (${document.url}) -> ${path.relative(process.cwd(), OUTPUT)} ` +
      `(${(fs.statSync(OUTPUT).size / 1024).toFixed(0)} KiB) + ` +
      `${(fs.statSync(RUNTIME_OUTPUT).size / 1024).toFixed(0)} KiB runtime subset`,
  );
}

main();
