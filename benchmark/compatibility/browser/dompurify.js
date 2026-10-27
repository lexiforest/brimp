import DOMPurify from 'dompurify';
import {check} from './check.js';

globalThis.runWorkload = async () => {
  const dirty = '<p>Hello <strong>world</strong><script>alert(1)</script><img src="x" onerror="alert(2)"><a href="javascript:alert(3)">link</a></p>';
  check(DOMPurify.isSupported === true, 'DOMPurify must support the DOM, not silently return input');
  const clean = DOMPurify.sanitize(dirty);
  const document = new DOMParser().parseFromString(clean, 'text/html');
  check(!document.querySelector('script,[onerror]'), 'active script and event handler must be removed');
  check(!document.querySelector('a').hasAttribute('href'), 'javascript URL must be removed');
  check(document.querySelector('strong').textContent === 'world', 'safe markup must survive');
  return {clean, text: document.body.textContent};
};
