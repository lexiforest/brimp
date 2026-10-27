// Plugins are explicit per workload. In particular, detectors get the default
// runtime, without graphics mocks or identity overrides intended to affect them.
export const workloads = [
  {id: 'todomvc-react', kind: 'application', source: 'todomvc', plugins: []},
  {id: 'todomvc-vue', kind: 'application', source: 'todomvc', plugins: []},
  {id: 'jquery', kind: 'library', source: 'jquery', plugins: []},
  {id: 'htmx', kind: 'library', source: 'htmx.org', plugins: []},
  {id: 'lit', kind: 'library', source: 'lit', plugins: []},
  {id: 'dompurify', kind: 'library', source: 'dompurify', plugins: []},
  {id: 'readability', kind: 'library', source: '@mozilla/readability', plugins: []},
  {id: 'axios', kind: 'library', source: 'axios', plugins: []},
  {id: 'dexie', kind: 'library', source: 'dexie', plugins: []},
  {id: 'jose-extended', kind: 'library', source: 'jose', plugins: []},
  {id: 'jose', kind: 'library', source: 'jose', plugins: []},
  {id: 'defuddle', kind: 'library', source: 'defuddle', plugins: []},
  {id: 'modernizr', kind: 'diagnostic', source: 'modernizr', plugins: []},
  {id: 'botd', kind: 'diagnostic', source: '@fingerprintjs/botd', plugins: []},
  {id: 'creepjs', kind: 'diagnostic', source: 'creepjs', plugins: []},
];

export const modernizrFeatures = [
  'cookies', 'storage/localstorage', 'storage/sessionstorage',
  'indexeddb', 'workers/webworkers', 'serviceworker',
  'websockets', 'network/fetch', 'es6/promises', 'es6/collections',
  'dom/mutationObserver', 'canvas', 'webgl',
];

// Explicit configurations; diagnostic checks always use the default runtime.
export const configurations = {default: []};
export const regressionWorkloads = new Set(['jquery', 'readability', 'axios', 'defuddle', 'todomvc-vue', 'dompurify', 'jose', 'jose-extended', 'todomvc-react', 'htmx', 'lit']);
export function workloadConfigurations(workload) {
  return ['default'];
}
export function isRegression(row) {
  return row.kind !== 'diagnostic' && regressionWorkloads.has(row.id) && row.configuration === 'default' && row.comparisons.brimp !== 'matched';
}
