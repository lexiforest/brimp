import {createSafariSession} from './webdriver.mjs';

// Shared by field-contract tests so the same assertions run in both runtimes.
export async function createSession(options = {}) {
  const browser = process.env.COVERAGE_BROWSER || 'brimp';
  if (browser === 'safari') return createSafariSession();
  if (browser !== 'brimp') throw Error('COVERAGE_BROWSER must be brimp or safari');
  // Plugins configure Brimp only; Safari supplies its native browser APIs.
  return (await import('../../bindings/node/index.js')).createSession(options);
}
