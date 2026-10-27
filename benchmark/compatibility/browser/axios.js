import axios from 'axios';
import {check} from './check.js';

globalThis.runWorkload = async () => {
  const client = axios.create({adapter: 'xhr', timeout: 3000});
  const json = await client.post('/api/echo', {answer: 42});
  check(json.data.body === '{"answer":42}' && json.data.method === 'POST', 'Axios must serialize a JSON POST');
  const form = await client.post('/api/echo', new URLSearchParams({name: 'Ada Lovelace'}));
  check(form.data.body === 'name=Ada+Lovelace', 'Axios must serialize form data');
  const binary = await client.get('/api/bytes', {responseType: 'arraybuffer'});
  const bytes = Array.from(new Uint8Array(binary.data));
  check(bytes.join() === '0,127,255', 'Axios must decode binary responses');
  const controller = new AbortController();
  const pending = client.get('/api/slow', {signal: controller.signal});
  setTimeout(() => controller.abort(), 20);
  let cancelled = false;
  try { await pending; } catch (error) { cancelled = axios.isCancel(error); }
  check(cancelled, 'Axios must report cancellation');
  return {json: json.data.body, form: form.data.body, bytes, cancelled};
};
