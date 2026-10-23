import {fork} from 'node:child_process';

// Native crashes and infinite JS must not terminate or wedge the whole suite.
export function executeWorker(script, args, {timeout = 60000, grace = 2000, initial = {}} = {}) {
  return new Promise(resolveRun => {
    const child = fork(script, args, {stdio: ['ignore', 'pipe', 'pipe', 'ipc'], detached: process.platform !== 'win32'});
    let partial = {...initial, status: 'setup_error'};
    let completed, expired = false, output = '', killer;
    const capture = chunk => { output = (output + chunk).slice(-16000); };
    child.stdout.on('data', capture);
    child.stderr.on('data', capture);
    child.on('message', message => {
      if (message.type === 'progress') partial = message.result;
      if (message.type === 'result') completed = message.result;
      if (message.type === 'cleanup_error' && completed) completed.cleanupError = message.message;
    });
    const killGroup = () => {
      try {
        if (process.platform === 'win32') child.kill('SIGKILL');
        else process.kill(-child.pid, 'SIGKILL');
      } catch (error) { if (error.code !== 'ESRCH') capture(String(error)); }
    };
    const timer = setTimeout(() => {
      expired = true;
      // Allow cleanup first, then enforce a second unconditional deadline for
      // native hangs or uncooperative cleanup.
      child.kill('SIGTERM');
      killer = setTimeout(killGroup, grace);
    }, timeout);
    child.on('error', error => { partial.failure = {message: String(error)}; });
    child.on('close', (code, signal) => {
      clearTimeout(timer);
      clearTimeout(killer);
      const result = completed ?? partial;
      if (expired || code !== 0) {
        result.status = expired ? 'timeout' : 'crash';
        result.failure = {message: expired ? `Worker exceeded ${timeout / 1000}s deadline (including cleanup)` : `Worker exited with code ${code}, signal ${signal}`};
      } else if (!completed) {
        result.status = 'crash';
        result.failure = {message: 'Worker exited without a result'};
      }
      if (result.cleanupError) {
        result.status = 'cleanup_error';
        result.failure = {message: result.cleanupError};
      }
      result.process = {code, signal, output};
      resolveRun(result);
    });
  });
}
