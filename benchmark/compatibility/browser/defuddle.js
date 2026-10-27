import Defuddle from 'defuddle/full';
import {check} from './check.js';

globalThis.runWorkload = async () => {
  // Browser entry, not Brimp's extraction wrapper or defuddle/node.
  const result = new Defuddle(document, {separateMarkdown: true, useAsync: false}).parse();
  check(result.title === 'The quiet observatory', 'Defuddle must extract the title');
  check(result.content.includes('the telescope recorded the first clear image'), 'Defuddle must preserve article content');
  check(!result.content.includes('Subscribe to our newsletter'), 'Defuddle must remove clutter');
  check(typeof result.contentMarkdown === 'string' && result.contentMarkdown.includes('telescope'), 'Defuddle must produce Markdown');
  return {title: result.title, author: result.author, content: result.content, markdown: result.contentMarkdown};
};
