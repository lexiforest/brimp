'use strict';
const { existsSync, readFileSync, mkdirSync, mkdtempSync, renameSync, rmSync } = require('node:fs');
const { createHash } = require('node:crypto');
const { homedir } = require('node:os');
const { join } = require('node:path');
const { execFileSync } = require('node:child_process');

exports.executablePath = function executablePath() {
  if (process.platform === 'linux') {
    const glibc = process.report.getReport().header.glibcVersionRuntime;
    const [major, minor] = (glibc || '0.0').split('.').map(Number);
    if (major < 2 || (major === 2 && minor < 39)) {
      throw new Error('@brimp/webkit requires Linux with glibc 2.39 or newer');
    }
  }
  const name = `@brimp/webkit-${process.platform}-${process.arch}`;
  let metadataPath;
  try {
    metadataPath = require.resolve(`${name}/runtime.json`);
  } catch (cause) {
    throw new Error(
      `WebKit is unavailable for ${process.platform}/${process.arch}. Install optional dependencies for @brimp/webkit.`,
      { cause },
    );
  }
  const metadata = JSON.parse(readFileSync(metadataPath, 'utf8'));
  const archive = require.resolve(`${name}/runtime.tar.xz`);
  const cache = join(process.env.XDG_CACHE_HOME || join(homedir(), '.cache'), 'brimp', 'webkit');
  const destination = join(cache, metadata.sha256);
  const executable = join(destination, 'runtime', 'brimp-webkit');
  if (existsSync(executable)) return executable;
  if (createHash('sha256').update(readFileSync(archive)).digest('hex') !== metadata.sha256) {
    throw new Error('The bundled WebKit runtime checksum does not match');
  }
  mkdirSync(cache, { recursive: true });
  const temporary = mkdtempSync(join(cache, 'extract-'));
  try {
    execFileSync('tar', ['-xf', archive, '-C', temporary]);
    if (!existsSync(join(temporary, 'runtime', 'brimp-webkit'))) {
      throw new Error('The WebKit runtime has no launcher');
    }
    try {
      renameSync(temporary, destination);
    } catch (error) {
      if (!['EEXIST', 'ENOTEMPTY'].includes(error.code) || !existsSync(executable)) throw error;
    }
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
  return executable;
};
