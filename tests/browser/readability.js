import { Readability } from '@mozilla/readability';
import { check } from './check.js';

globalThis.runWorkload = async () => {
  const result = new Readability(document.cloneNode(true)).parse();
  check(
    result && result.title === 'The quiet observatory',
    'Readability must extract the article title',
  );
  check(
    result.textContent.includes('the telescope recorded the first clear image'),
    'Readability must preserve the article body',
  );
  check(
    !result.textContent.includes('Subscribe to our newsletter'),
    'Readability must exclude navigation clutter',
  );
  return {
    title: result.title,
    text: result.textContent.replace(/\s+/g, ' ').trim(),
    byline: result.byline,
  };
};
