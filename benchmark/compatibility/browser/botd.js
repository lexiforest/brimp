import {load} from '@fingerprintjs/botd';
import {check} from './check.js';

globalThis.runWorkload = async () => {
  const detector = await load({monitoring: false});
  const detection = detector.detect();
  check(typeof detection?.bot === 'boolean', 'BotD must return a detection result');
  check(!detection.bot || typeof detection.botKind === 'string', 'positive detection must include its category');
  return {detection};
};
