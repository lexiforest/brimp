import { execFileSync } from 'node:child_process';
import { mkdirSync, readdirSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { resolve } from 'node:path';
import sharp from 'sharp';

const docs = fileURLToPath(new URL('..', import.meta.url));
const run = (program, args, options = {}) => execFileSync(program, args, {cwd: docs, stdio: 'inherit', ...options});
run(process.env.PYTHON || 'python3', ['-m', 'mkdocs', 'build', '--strict', '--config-file', 'mkdocs.yml']);
run(process.execPath, ['node_modules/typedoc/bin/typedoc', '--options', 'typedoc.json']);
const cli = run('cargo', ['run', '--locked', '--quiet', '--manifest-path', 'tools/cli-reference/Cargo.toml'], {stdio: ['ignore', 'pipe', 'inherit'], encoding: 'utf8'});
writeFileSync(resolve(docs, 'src/content/docs/api/cli.md'), cli.replace(/[\t ]+$/gm, ''));
mkdirSync(resolve(docs, 'public/diagrams'), {recursive: true});
for (const file of readdirSync(resolve(docs, 'diagrams')).filter(name => name.endsWith('.svg'))) {
  await sharp(resolve(docs, 'diagrams', file), {density: 144})
    .png().toFile(resolve(docs, 'public/diagrams', file.replace(/\.svg$/, '.png')));
}
