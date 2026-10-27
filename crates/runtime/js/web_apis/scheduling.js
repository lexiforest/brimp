(({ global }) => {
  const priorities = ['user-blocking', 'user-visible', 'background'];
  const priority = (value) => {
    if (!priorities.includes(value)) throw new TypeError('Invalid task priority');
    return value;
  };
  const signalPriorities = new WeakMap();
  class TaskPriorityChangeEvent extends Event {
    constructor(type, options = {}) {
      super(type, options);
      this.previousPriority = priority(options.previousPriority);
    }
  }
  class TaskSignal extends AbortSignal {
    get priority() {
      return signalPriorities.get(this);
    }
  }
  class TaskController extends AbortController {
    constructor(options = {}) {
      super();
      Object.setPrototypeOf(this.signal, TaskSignal.prototype);
      signalPriorities.set(this.signal, priority(options.priority ?? 'user-visible'));
    }
    setPriority(value) {
      value = priority(value);
      const previousPriority = this.signal.priority;
      if (value === previousPriority) return;
      signalPriorities.set(this.signal, value);
      this.signal.dispatchEvent(
        new TaskPriorityChangeEvent('prioritychange', { previousPriority }),
      );
    }
  }
  const schedule = (callback, delay, rank) =>
    __callHost('scheduleTask', null, callback, delay, rank);
  const cancel = (id) => __callHost('clearTimeout', null, id);
  class Scheduler {
    postTask(callback, options = {}) {
      return new Promise((resolve, reject) => {
        if (typeof callback !== 'function') throw new TypeError('Callback required');
        const signal = options.signal;
        if (signal && !(signal instanceof AbortSignal)) throw new TypeError('Invalid signal');
        if (signal?.aborted) {
          reject(signal.reason);
          return;
        }
        const delay = Number(options.delay ?? 0);
        if (!Number.isFinite(delay) || delay < 0) throw new TypeError('Invalid delay');
        const fixed = options.priority === undefined ? undefined : priority(options.priority);
        const rank = () => priorities.indexOf(fixed ?? signal?.priority ?? 'user-visible');
        const cleanup = () => {
          signal?.removeEventListener('abort', abort);
          signal?.removeEventListener('prioritychange', change);
        };
        const id = schedule(
          () => {
            cleanup();
            try {
              resolve(callback());
            } catch (error) {
              reject(error);
            }
          },
          delay,
          rank(),
        );
        const abort = () => {
          cancel(id);
          cleanup();
          reject(signal.reason);
        };
        const change = () => __callHost('taskPriority', null, id, rank());
        signal?.addEventListener('abort', abort, { once: true });
        if (fixed === undefined) signal?.addEventListener('prioritychange', change);
      });
    }
    yield() {
      return this.postTask(() => undefined);
    }
  }
  const idle = new Set();
  function requestIdleCallback(callback, options = {}) {
    if (typeof callback !== 'function') throw new TypeError('Callback required');
    const timeout = options.timeout === undefined ? Infinity : Number(options.timeout);
    if (Number.isNaN(timeout) || timeout < 0) throw new TypeError('Invalid timeout');
    const start = performance.now();
    const id = schedule(
      () => {
        idle.delete(id);
        callback({
          didTimeout: performance.now() - start >= timeout,
          timeRemaining: () => 0,
        });
      },
      0,
      3,
    );
    idle.add(id);
    return id;
  }
  function cancelIdleCallback(id) {
    idle.delete(id);
    cancel(id);
  }
  for (const [name, value] of Object.entries({
    Scheduler,
    TaskController,
    TaskSignal,
    TaskPriorityChangeEvent,
    requestIdleCallback,
    cancelIdleCallback,
    scheduler: new Scheduler(),
  }))
    Object.defineProperty(global, name, { value, writable: true, configurable: true });
})({ global: globalThis });
