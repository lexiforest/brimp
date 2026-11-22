// Returned to Inspector as an object reference; never installed on the page global.
function createController(bridge, poll) {
  const pending = new Map();
  const pause = () => new Promise((resolve) => setTimeout(resolve, 10));
  return async function (command) {
    if (command.op === 'cancel') {
      pending.get(command.id)?.reject(new Error('cancelled: operation cancelled'));
      return null;
    }
    let timer;
    const state = { cancelled: false, cleanup: () => {} };
    const aborted = new Promise((_, reject) => {
      state.reject = (error) => {
        state.cancelled = true;
        state.cleanup();
        reject(error);
      };
      if (command.timeout)
        timer = setTimeout(
          () => state.reject(new Error('timeout: operation timed out')),
          command.timeout,
        );
    });
    pending.set(command.id, state);
    try {
      return await Promise.race([
        aborted,
        (async () => {
          if (command.op === 'evaluate') {
            const value = await (0, eval)(command.source);
            if (state.cancelled) return null;
            return command.handle
              ? { type: 'reference', value: JSON.parse(bridge('{"op":"retain"}', { value })) }
              : bridge.codec.encode(value);
          }
          if (command.op === 'bridge') {
            const result = bridge(JSON.stringify(command.command));
            state.cleanup = () => result.cancel?.();
            while (!result.done && !state.cancelled) await pause();
            if (state.cancelled) return null;
            if ('error' in result) throw new Error(result.error);
            return result.wire;
          }
          if (command.op === 'query') {
            const root = command.token
              ? bridge(JSON.stringify({ op: 'get', token: command.token })).value
              : document;
            const nodes = command.all
              ? [...root.querySelectorAll(command.selector)]
              : [root.querySelector(command.selector)].filter(Boolean);
            return {
              type: 'array',
              value: nodes.map((value) => ({
                type: 'reference',
                value: JSON.parse(bridge('{"op":"retain"}', { value })),
              })),
            };
          }
          if (command.op !== 'automate') throw new Error('unsupported: unknown operation');
          const input = command.command;
          const polling = {};
          while (!state.cancelled) {
            const result = poll(JSON.stringify(input), polling, (target) => {
              const rect = target.getBoundingClientRect();
              const view = target.ownerDocument.defaultView;
              if (
                rect.top < 0 ||
                rect.left < 0 ||
                rect.bottom > view.innerHeight ||
                rect.right > view.innerWidth
              )
                target.scrollIntoView({ block: 'center', inline: 'center' });
            });
            if (result.error) throw new Error('invalid_input: ' + result.error);
            if (result.done) {
              if (input.op !== 'input') return bridge.codec.encode(result.value);
              const target = result.target;
              if (['fill', 'type', 'press', 'focus'].includes(input.input.action)) target.focus();
              if (input.input.action === 'fill') {
                if (typeof target.select === 'function') target.select();
                else {
                  const range = target.ownerDocument.createRange();
                  range.selectNodeContents(target);
                  const selection = target.ownerDocument.getSelection();
                  selection.removeAllRanges();
                  selection.addRange(range);
                }
              }
              const rect = target.getBoundingClientRect();
              let x = rect.x + rect.width / 2;
              let y = rect.y + rect.height / 2;
              for (
                let view = target.ownerDocument.defaultView;
                view !== window;
                view = view.parent
              ) {
                const frame = view.frameElement;
                if (!frame) throw new Error('unsupported: cross-origin frame coordinates');
                const rect = frame.getBoundingClientRect();
                x += rect.x + frame.clientLeft;
                y += rect.y + frame.clientTop;
              }
              return bridge.codec.encode([x, y]);
            }
            await pause();
          }
          return null;
        })(),
      ]);
    } finally {
      clearTimeout(timer);
      pending.delete(command.id);
    }
  };
}
