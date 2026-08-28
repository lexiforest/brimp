const pending = state.pendingPromise;
delete state.pendingPromise;
if (pending.rejected) throw pending.value;
const value = pending.value;
