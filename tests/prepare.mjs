import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { cp, mkdir, readFile, readdir, rename, rm, writeFile } from 'node:fs/promises';
import { dirname, join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';
import { build } from 'esbuild';
import { workloads } from './catalog.mjs';

const root = dirname(fileURLToPath(import.meta.url));
const sha256 = (bytes) => createHash('sha256').update(bytes).digest('hex');
const json = async (path) => JSON.parse(await readFile(path, 'utf8'));
const sources = await json(join(root, 'sources.json'));
const lock = await json(join(root, 'package-lock.json'));
const staging = join(root, '.cache', 'build');
await rm(staging, { recursive: true, force: true });
await mkdir(staging, { recursive: true });

for (const [name, source] of Object.entries(sources)) {
  const archivePath = join(root, '.cache', `${name}.tar.gz`);
  let archive;
  try {
    archive = await readFile(archivePath);
  } catch (error) {
    if (error.code !== 'ENOENT') throw error;
    console.log(`Downloading ${name}@${source.revision}`);
    const response = await fetch(source.archive, { signal: AbortSignal.timeout(120000) });
    if (!response.ok) throw new Error(`${source.archive}: HTTP ${response.status}`);
    archive = Buffer.from(await response.arrayBuffer());
  }
  if (sha256(archive) !== source.sha256)
    throw new Error(`${name}: source archive SHA-256 mismatch`);
  await writeFile(archivePath, archive);
  const destination = join(root, '.cache', name);
  await rm(destination, { recursive: true, force: true });
  await mkdir(destination, { recursive: true });
  execFileSync('tar', ['-xzf', archivePath, '--strip-components=1', '-C', destination]);
  const notices = join(staging, 'notices', name);
  await mkdir(notices, { recursive: true });
  for (const file of await readdir(destination)) {
    if (/^(license|copying|notice|trademarks)/i.test(file))
      await cp(join(destination, file), join(notices, file));
  }
}

for (const framework of ['react', 'vue']) {
  const destination = join(staging, `todomvc-${framework}`);
  await cp(join(root, '.cache/todomvc/examples', framework, 'dist'), destination, {
    recursive: true,
  });
  // Preserve the upstream build's dependency lock and license context too.
  await cp(
    join(root, '.cache/todomvc/examples', framework, 'package-lock.json'),
    join(staging, 'notices', `todomvc-${framework}-package-lock.json`),
  );
}

for (const workload of workloads.filter((id) => !id.startsWith('todomvc-'))) {
  const destination = join(staging, workload);
  await mkdir(destination, { recursive: true });
  await build({
    absWorkingDir: root,
    entryPoints: [`browser/${workload}.js`],
    outfile: join(destination, 'workload.js'),
    bundle: true,
    platform: 'browser',
    format: 'iife',
    target: 'es2022',
    sourcemap: true,
    legalComments: 'eof',
    metafile: true,
    write: true,
  }).then(async (result) => {
    await writeFile(
      join(destination, 'build-inputs.json'),
      JSON.stringify(result.metafile, null, 2) + '\n',
    );
  });
  const fixture = ['readability', 'defuddle'].includes(workload)
    ? 'article'
    : workload === 'htmx'
      ? 'htmx'
      : null;
  if (fixture) await cp(join(root, 'fixtures', `${fixture}.html`), join(destination, 'index.html'));
  else
    await writeFile(
      join(destination, 'index.html'),
      '<!doctype html><html><head><meta charset="utf-8"><script defer src="/workload.js"></script></head><body></body></html>\n',
    );
}

// Preserve installed dependency notices, including transitive browser code.
for (const packagePath of Object.keys(lock.packages).filter((path) =>
  path.startsWith('node_modules/'),
)) {
  const installed = join(root, packagePath);
  let files;
  try {
    files = await readdir(installed);
  } catch (error) {
    if (error.code === 'ENOENT' && lock.packages[packagePath].optional) continue;
    throw error;
  }
  const destination = join(staging, 'notices', packagePath);
  await mkdir(destination, { recursive: true });
  for (const file of files.filter(
    (name) => /^(license|copying|notice)(\.|$|-)/i.test(name) || name === 'package.json',
  )) {
    await cp(join(installed, file), join(destination, file), { recursive: true });
  }
}

async function hashes(directory) {
  const result = {};
  for (const entry of (await readdir(directory, { withFileTypes: true })).sort((a, b) =>
    a.name.localeCompare(b.name),
  )) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) Object.assign(result, await hashes(path));
    else result[relative(staging, path)] = sha256(await readFile(path));
  }
  return result;
}
const manifest = {
  schema: 1,
  sources,
  packages: lock.packages,
  unchangedArtifacts: {
    'todomvc-react': 'todomvc/examples/react/dist',
    'todomvc-vue': 'todomvc/examples/vue/dist',
  },
  packageLockSha256: sha256(await readFile(join(root, 'package-lock.json'))),
  build: {
    format: 'iife',
    platform: 'browser',
    target: 'es2022',
    esbuild: lock.packages['node_modules/esbuild'].version,
  },
  files: await hashes(staging),
};
manifest.inputs = {};
for (const directory of ['browser', 'fixtures']) {
  for (const file of (await readdir(join(root, directory))).sort()) {
    const path = `${directory}/${file}`;
    manifest.inputs[path] = sha256(await readFile(join(root, path)));
  }
}
for (const path of ['prepare.mjs', 'catalog.mjs', 'sources.json'])
  manifest.inputs[path] = sha256(await readFile(join(root, path)));
await writeFile(join(staging, 'manifest.json'), JSON.stringify(manifest, null, 2) + '\n');
await rm(join(root, 'dist'), { recursive: true, force: true });
await rename(staging, join(root, 'dist'));
console.log(
  `Prepared ${workloads.length} workloads; provenance and asset hashes: dist/manifest.json`,
);
