import { build } from 'esbuild';
import { copyFile, readFile, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
process.chdir(fileURLToPath(new URL('.', import.meta.url)));
await build({
  absWorkingDir: process.cwd(),
  entryPoints: ['core.js'],
  outfile: 'core.bundle.js',
  bundle: true,
  platform: 'browser',
  format: 'iife',
  legalComments: 'eof',
  plugins: [
    {
      name: 'realm-private-core-js-store',
      setup(builder) {
        // One bundled core-js instance owns this store. Preserve its shared-module
        // lifetime without registering dependency metadata on the page's global.
        builder.onLoad({ filter: /[/\\]core-js-pure[/\\]internals[/\\]shared-store\.js$/ }, () => ({
          contents: 'module.exports = {};',
          loader: 'js',
        }));
      },
    },
  ],
});
const storage = await build({
  absWorkingDir: process.cwd(),
  entryPoints: ['storage.js'],
  bundle: true,
  platform: 'browser',
  format: 'iife',
  globalName: '__BrimpStorage',
  write: false,
  legalComments: 'eof',
  plugins: [
    {
      name: 'session-indexeddb-backend',
      setup(builder) {
        builder.onLoad(
          {
            filter: /[/\\]fake-indexeddb[/\\]build[/\\]esm[/\\]FDB(Factory|Transaction)\.js$/,
          },
          async ({ path }) => {
            let contents = await readFile(path, 'utf8');
            const required = path.endsWith('FDBFactory.js')
              ? [
                  'queueTask(() => {\n      openDatabase(',
                  'queueTask(() => {\n      deleteDatabase(',
                  '(err, connection) => {\n        if (err)',
                  '(err, oldVersion) => {\n        if (err)',
                  '  open(name, version) {',
                  '  deleteDatabase(name) {',
                  '  databases() {',
                ]
              : [
                  '  _start() {',
                  '  _abort(errName) {',
                  '      this._markFinished();\n      if (!this.error)',
                ];
            for (const needle of required)
              if (contents.split(needle).length !== 2)
                throw Error(`Review pinned IndexedDB adapter: ${path}: ${needle}`);
            const adapter = fileURLToPath(new URL('./storage-idb.js', import.meta.url));
            if (path.endsWith('FDBFactory.js')) {
              contents =
                `import {beginOpen,endOpen,databases,validateOrigin} from ${JSON.stringify(adapter)};\n` +
                contents;
              contents = contents.replaceAll(
                'queueTask(() => {\n      openDatabase(',
                'queueTask(async () => {\n      const lease = await beginOpen(this, name);\n      openDatabase(',
              );
              contents = contents.replaceAll(
                'queueTask(() => {\n      deleteDatabase(',
                'queueTask(async () => {\n      const lease = await beginOpen(this, name);\n      deleteDatabase(',
              );
              contents = contents.replace(
                '(err, connection) => {\n        if (err)',
                '(err, connection) => {\n        endOpen(this, name, lease);\n        if (err)',
              );
              contents = contents.replace(
                '(err, oldVersion) => {\n        if (err)',
                '(err, oldVersion) => {\n        endOpen(this, name, lease, !err);\n        if (err)',
              );
              contents = contents.replace(
                '  open(name, version) {',
                '  open(name, version) {\n    validateOrigin();',
              );
              contents = contents.replace(
                '  deleteDatabase(name) {',
                '  deleteDatabase(name) {\n    validateOrigin();',
              );
              contents = contents.replace(
                /  databases\(\) \{[\s\S]*?\n  get \[Symbol\.toStringTag\]/,
                '  databases() { return Promise.resolve(databases()); }\n  get [Symbol.toStringTag]',
              );
            } else {
              contents =
                `import {beginTransaction,endTransaction} from ${JSON.stringify(adapter)};\n` +
                contents;
              contents = contents.replace(
                '  _start() {',
                '  _start() {\n    if (!beginTransaction(this)) return;',
              );
              contents = contents.replace(
                '  _abort(errName) {',
                '  _abort(errName) {\n    endTransaction(this, false);',
              );
              contents = contents.replace(
                '      this._markFinished();\n      if (!this.error)',
                '      if (!endTransaction(this, !this.error)) return;\n      this._markFinished();\n      if (!this.error)',
              );
            }
            return {
              contents,
              loader: 'js',
              resolveDir: fileURLToPath(
                new URL('./node_modules/fake-indexeddb/build/esm/', import.meta.url),
              ),
            };
          },
        );
      },
    },
  ],
});
await build({
  absWorkingDir: process.cwd(),
  entryPoints: ['input-color.js'],
  outfile: 'input-color.bundle.js',
  bundle: true,
  platform: 'browser',
  format: 'iife',
  globalName: '__inputColor',
  legalComments: 'eof',
  plugins: [
    {
      name: 'safari-color-math',
      setup(builder) {
        builder.onLoad(
          { filter: /[/\\]@csstools[/\\]color-helpers[/\\]dist[/\\]index\.mjs$/ },
          async ({ path }) => {
            const source = await readFile(path, 'utf8');
            // Use Safari's float conversions throughout the parser, including nested
            // relative colors and color-mix. Do not change the dependency on disk.
            const functions = [
              'multiplyMatrices',
              'lin_sRGB_channel',
              'gam_sRGB_channel',
              'lin_2020_channel',
              'gam_2020_channel',
              'Lab_to_XYZ',
            ];
            let contents = "import * as safari from './safari-color-math.js';\n" + source;
            for (const name of functions) {
              const pattern = new RegExp(`function ${name}\\([^)]*\\)\\{[^}]*\\}`, 'g');
              if ([...contents.matchAll(pattern)].length !== 1)
                throw Error(`Review pinned color helper adapter: ${name}`);
              contents = contents.replace(
                pattern,
                `function ${name}(...args){return safari.${name}(...args);}`,
              );
            }
            // Avoid a lossy HSL/HWB -> XYZ -> sRGB round trip in the input adapter.
            for (const name of ['HSL_to_sRGB', 'HWB_to_sRGB', 'sRGB_to_HSL', 'sRGB_to_Hue']) {
              if (!contents.includes(`function ${name}(`))
                throw Error(`Review pinned color helper export: ${name}`);
            }
            contents += '\nexport {HSL_to_sRGB, HWB_to_sRGB, sRGB_to_HSL, sRGB_to_Hue};\n';
            return { contents, loader: 'js', resolveDir: process.cwd() };
          },
        );
        builder.onLoad(
          { filter: /[/\\]@csstools[/\\]css-color-parser[/\\]dist[/\\]index\.mjs$/ },
          async ({ path }) => {
            let source = await readFile(path, 'utf8');
            // CSS color channels are floats in Safari, not signed 32-bit integers.
            // Retain actual infinities and extended values through parsing.
            if ((source.match(/2147483647/g) ?? []).length !== 34)
              throw Error('Review pinned color parser channel limits');
            source = source.replaceAll('2147483647', 'Infinity');
            const signature = /function convertToDestination\((\w+),(\w+)\)\{/g;
            if ([...source.matchAll(signature)].length !== 1)
              throw Error('Review pinned color parser conversion adapter');
            const contents =
              "import {directSRGBConversion} from './safari-color-conversion.js';\n" +
              source.replace(
                signature,
                (match, color, destination) =>
                  match +
                  `if(${color}.colorNotation!==${destination}){const direct=directSRGBConversion(${color},${destination});if(direct){${color}.colorNotation=${destination};${color}.channels=convertPowerlessComponentsToMissingComponents(direct,${destination});return ${color};}}`,
              );
            return { contents, loader: 'js', resolveDir: process.cwd() };
          },
        );
      },
    },
  ],
});
await writeFile(
  'storage.bundle.js',
  '// Generated by js/vendor/build.mjs. Edit vendor/storage*.js.\n(() => {\n' +
    storage.outputFiles[0].text +
    '\nconst host = globalThis.__brimpStorageHost; delete globalThis.__brimpStorageHost; __BrimpStorage.install({global:globalThis, host});\n})();\n',
);
const licenses = {
  'core-js-pure': 'LICENSE',
  xpath: 'LICENSE',
  'fake-indexeddb': 'LICENSE',
  'web-streams-polyfill': 'LICENSE',
};
for (const [name, file] of Object.entries(licenses))
  await copyFile(`node_modules/${name}/${file}`, `${name}.LICENSE`);
for (const name of [
  'color-helpers',
  'css-calc',
  'css-color-parser',
  'css-parser-algorithms',
  'css-tokenizer',
]) {
  await copyFile(`node_modules/@csstools/${name}/LICENSE.md`, `csstools-${name}.LICENSE`);
}
const sha256 = (bytes) => createHash('sha256').update(bytes).digest('hex');
await writeFile(
  'manifest.json',
  JSON.stringify(
    {
      packages: JSON.parse(await readFile('package.json')).dependencies,
      lockSha256: sha256(await readFile('package-lock.json')),
      bundleSha256: sha256(await readFile('core.bundle.js')),
      storageSha256: sha256(await readFile('storage.bundle.js')),
      inputColorSha256: sha256(await readFile('input-color.bundle.js')),
    },
    null,
    2,
  ) + '\n',
);
