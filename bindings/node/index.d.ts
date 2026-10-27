/// <reference types="node" />
export type Headers = Record<string, string> | [string, string][];
export type JSONValue = null | boolean | number | string | JSONValue[] | { [key: string]: JSONValue };
export interface IdentityOptions { userAgent?: string; languages?: string[]; platform?: string; }
/** Partial interface-owned values, optionally produced by `brimp profile capture`. */
export interface BrowserProfile {
  format?: 'brimp-profile';
  version?: 1;
  values?: Record<string, Record<string, JSONValue>>;
  capture?: JSONValue;
}
export interface DiagnosticsOptions { mode?: 'api' | 'recursive'; path?: string; include?: string[]; capacity?: number; stack?: boolean; }
export interface TraceRecord { sequence: number; pageId: number; realmId: number; elapsedMs: number; kind: string; path: string; outcome: string; stack: string | null; details: {layer?: 'recursive'; note?: string; arguments?: JSONValue[]; argumentCount?: number; target?: string | null; property?: string | null} | null; }
export interface DiagnosticCoverage { enabled: boolean; mode: 'api' | 'recursive'; include: string[]; limitations: string[]; }
export interface DiagnosticBatch { records: TraceRecord[]; dropped: number; writeError: string | null; coverage: DiagnosticCoverage; }
export interface DependencyUsage {
  layer: 'api' | 'native' | 'reflection' | 'recursive'; path: string; kind: string;
  target: string | null; property: string | null; count: number; throws: number;
}
export interface DiagnosticSummary {
  dependencies: DependencyUsage[]; retained: number; dropped: number;
  writeError: string | null; coverage: DiagnosticCoverage;
}
export interface SessionOptions {
  /** Persist origin localStorage, CacheStorage and IndexedDB in this file. */
  storagePath?: string | null;
  clock?: { mode?: 'system' | 'logical'; epochMs?: number };
  diagnostics?: DiagnosticsOptions;

  identity?: IdentityOptions;
  profile?: BrowserProfile;
  /** Omit for browser defaults; an explicit list replaces them. [] selects core only. */
  plugins?: PluginDescriptor[];
  headers?: Headers;
  /** Milliseconds. Default 30000. */
  timeout?: number;
  proxy?: string;
  auth?: [string, string];
}
export interface RequestOptions {
  headers?: Headers;
  body?: string | Buffer | Uint8Array;
  /** Milliseconds. Inherits the session default. */
  timeout?: number;
  allowRedirects?: boolean;
  maxRedirects?: number;
  auth?: [string, string];
  signal?: AbortSignal;
}
export interface Cookie {
  name: string; value: string; domain: string; path: string;
  hostOnly: boolean; httpOnly: boolean; secure: boolean;
  expires: number | null; sameSite: string | null;
}
export interface RequestRecord {
  method: string; url: string; headers: [string, string][]; body: Buffer | null;
}
export interface Redirect {
  statusCode: number; reason: string; url: string; headers: [string, string][]; request: RequestRecord;
}
export class RuntimeError extends Error { readonly code: string; }
export class JavaScriptError extends RuntimeError {}
export class StaleHandle extends RuntimeError {}
export class Closed extends RuntimeError {}
export class Timeout extends RuntimeError {}
export class Reentry extends RuntimeError {}
/** Create an in-process browser session. Close it to release all owned pages. */
export function createSession(options?: SessionOptions): Promise<Session>;
declare class Requests {
  request(method: string, url: string, options?: RequestOptions): Promise<Page>;
  get(url: string, options?: RequestOptions): Promise<Page>;
  post(url: string, options?: RequestOptions): Promise<Page>;
  put(url: string, options?: RequestOptions): Promise<Page>;
  patch(url: string, options?: RequestOptions): Promise<Page>;
  delete(url: string, options?: RequestOptions): Promise<Page>;
  head(url: string, options?: RequestOptions): Promise<Page>;
  options(url: string, options?: RequestOptions): Promise<Page>;
}
/** Shared cookies, request defaults and plugins. Request methods create new pages. */
export class Session extends Requests {
  private constructor();
  newPage(): Promise<Page>;
  setCookie(url: string, name: string, value: string): void;
  readonly cookies: Cookie[];
  clearCookies(): void;
  close(): Promise<void>;
}
export interface StreamingResponse { status: number; url: string; headers: [string,string][]; body: import("node:stream").Readable; cancel(): void; }
/** A reusable browser realm and DOM. Navigation invalidates previous realm handles. */
export interface ProfileField {
  default: unknown;
  type: string;
  [constraint: string]: unknown;
}
export interface ProfileContract {
  format: 'brimp-profile-contract';
  version: 1;
  interfaces: Array<{
    scope: string;
    owner: string;
    version: number;
    fields: Record<string, ProfileField>;
    probe: string | null;
    notes: string | null;
  }>;
}
export interface ProfileReport {
  defaultsUsed: string[];
  capture: unknown;
}
export class Page extends Requests {
  profileContract(): Promise<ProfileContract>;
  profileReport(): Promise<ProfileReport>;
  advanceTime(milliseconds: number, options?: {maxTasks?: number}): Promise<{now: number; tasks: number}>;
  runPendingTasks(options?: {maxTasks?: number}): Promise<{now: number; tasks: number}>;
  /** Consume retained records and the dropped count. JSONL output is unaffected. */
  takeDiagnostics(): Promise<DiagnosticBatch>;
  /** Inspect retained records without consuming them or the dropped count. */
  peekDiagnostics(): Promise<DiagnosticBatch>;
  /** Group retained observations. This does not imply complete dependency coverage. */
  summarizeDiagnostics(): Promise<DiagnosticSummary>;

  stream(method: string, url: string, options?: RequestOptions): Promise<StreamingResponse>;
  private constructor();
  readonly response: Response | null;
  readonly statusCode: number;
  readonly content: Buffer;
  readonly text: string;
  readonly url: string;
  readonly headers: [string, string][];
  json(): JSONValue;
  /** Evaluate browser JavaScript, await Promises, and copy rich values, including bigint and binary buffers. */
  evaluate(source: string): Promise<CopiedValue>;
  evaluateHandle(source: string): Promise<JSHandle>;
  querySelector(selector: string): Promise<ElementHandle | null>;
  querySelectorAll(selector: string): Promise<ElementHandle[]>;
  setIdentity(identity: IdentityOptions): Promise<void>;
  addInitScript(source: string): Promise<void>;
  click(selector: string): Promise<void>;
  hover(selector: string): Promise<void>;
  tap(selector: string): Promise<void>;
  focus(selector: string): Promise<void>;
  typeText(selector: string, text: string): Promise<void>;
  fill(selector: string, text: string): Promise<void>;
  /** Modifier bits: Alt=1, Control=2, Meta=4, Shift=8. */
  press(selector: string, key: string, options?: {modifiers?: number}): Promise<void>;
  dragTo(source: string, destination: string): Promise<void>;
  setInputFiles(selector: string, files: FileUpload[]): Promise<void>;
  extract(options?: ExtractionOptions): Promise<ExtractedDocument>;
  /** Serialize the current DOM, including page-script and host edits. */
  html(): Promise<string>;
  close(): Promise<void>;
}
/** Original HTTP response bytes and metadata. Use Page.html() for the live DOM. */
export class Response {
  private constructor();
  readonly statusCode: number;
  readonly reason: string;
  readonly url: string;
  readonly headers: [string, string][];
  readonly cookies: [string, string][];
  readonly history: Redirect[];
  readonly request: RequestRecord;
  /** Milliseconds. */
  readonly elapsed: number;
  readonly httpVersion: string | null;
  readonly downloadedBytes: number;
  readonly uploadedBytes: number;
  readonly headerBytes: number;
  readonly content: Buffer;
  readonly text: string;
  json(): JSONValue;
}
/** Rooted JavaScript value. Dispose it when finished; navigation makes it stale. */
export class JSHandle {
  protected constructor();
  /** Copy the value using the shared bridge; non-copyable values require handles. */
  value(): Promise<CopiedValue>;
  /** Explicit JSON serialization, with JavaScript JSON.stringify semantics. */
  jsonValue(): Promise<JSONValue>;
  /** Calls function(value, ...args) and awaits its result in the page realm. */
  call(functionSource: string, ...args: HostValue[]): Promise<JSHandle>;
  /** Invoke the held function with an optional receiver. */
  invoke(args?: HostValue[], options?: {thisArg?: HostValue}): Promise<JSHandle>;
  setProperty(name: string, value: HostValue): Promise<void>;
  deleteProperty(name: string): Promise<boolean>;
  property(name: string): Promise<JSHandle>;
  propertyNames(): Promise<string[]>;
  dispose(): Promise<void>;
}
export interface FileUpload { name: string; content: Uint8Array; mimeType?: string; lastModified?: number; }
export class ElementHandle extends JSHandle {
  focus(): Promise<void>;
  click(): Promise<void>;
  setInputFiles(files: FileUpload[]): Promise<void>;
  getAttribute(name: string): Promise<string | null>;
  setAttribute(name: string, value: string): Promise<null>;
  textContent(): Promise<string>;
  html(): Promise<string>;
  setTextContent(text: string): Promise<null>;
  remove(): Promise<null>;
  querySelector(selector: string): Promise<ElementHandle | null>;
  querySelectorAll(selector: string): Promise<ElementHandle[]>;
}

export type CopiedValue = undefined | null | boolean | number | bigint | string | ArrayBufferView | ArrayBuffer | CopiedValue[] | { [key: string]: CopiedValue };
export type HostValue = undefined | null | boolean | number | bigint | string | ArrayBufferView | ArrayBuffer | JSHandle | HostFunction | HostValue[] | { [key: string]: HostValue };
/** A registered plugin handler exposed to the page as a Promise-returning function. */
export class HostFunction { private constructor(); readonly name: string; }
export type HostHandler = (context: HostContext, ...args: HostValue[]) => HostValue | Promise<HostValue>;
export interface PluginDescriptor { name: string; source: string; handlers?: Record<string, HostHandler>; }
export interface PluginSessionScope { handlers?: Record<string, HostHandler>; dispose?: () => void | Promise<void>; }
export class Plugin implements PluginDescriptor {
  constructor(name: string, source: string, handlers?: Record<string, HostHandler>, options?: JSONValue, setup?: () => PluginSessionScope);
  static fromFile(path: string | URL, configuration: {name: string; handlers?: Record<string, HostHandler>; options?: JSONValue; setup?: () => PluginSessionScope}): Plugin;
  readonly name: string;
  readonly source: string;
  readonly handlers: Record<string, HostHandler>;
}
export class HostContext {
  private constructor();
  readonly sessionId: string;
  readonly url: string | null;
  readonly origin: string | null;
  readonly pageId: string;
  readonly realmId: string;
  readonly cancelled: boolean;
  function(name: string): HostFunction;
  emit(channel: string, value: HostValue): void;
}

export function canvasPlugin(options?: { rgba?: [number, number, number, number]; characterWidth?: number }): Plugin;
export function environmentPlugin(options?: Record<string, JSONValue>): Plugin;
export function permissionsPlugin(options?: Record<string, JSONValue>): Plugin;
export function devicesPlugin(options?: Record<string, JSONValue>): Plugin;
export function mediaPlugin(options?: Record<string, JSONValue>): Plugin;
export function workersPlugin(options?: Record<string, JSONValue>): Plugin;
export function trustedTypesPlugin(options?: Record<string, JSONValue>): Plugin;
export function extractionPlugin(): Plugin;
export interface ExtractionOptions { contentSelector?: string; removeImages?: boolean; language?: string; debug?: boolean; }
export interface ExtractedDocument {
  title: string; description: string; domain: string; favicon: string; image: string;
  language: string; parseTime: number; published: string; author: string; site: string;
  schemaOrgData: JSONValue; wordCount: number; content: string; contentMarkdown?: string;
  extractorType?: string; metaTags?: {name?: string; property?: string; content?: string}[];
}

export interface NetworkOptions {
  offline?: boolean;
  /** Milliseconds, before transport submission. */ latency?: number;
  bypassCache?: boolean;
  maxRetainedBytes?: number;
  maxRetainedBodyBytes?: number;
  /** Delay for fully buffered responses, not transport bandwidth throttling. */
  bufferedDownloadBytesPerSecond?: number;
}
export interface NetworkRequest extends RequestRecord { id: number; resourceType: string | null; }
export interface NetworkResponse { requestId: number; url: string; status: number; headers: [string,string][]; fromCache: boolean; }
export interface MutationRecord { kind: string; target: number; added: number[]; removed: number[]; attribute: string | null; oldValue: string | null; }
export interface PageEvents {
  console: {event: 'console'; level: string; message: string};
  error: {event: 'error'; message: string};
  lifecycle: {event: 'lifecycle'; name: string; url: string};
  mutation: {event: 'mutation'; record: MutationRecord};
  request: {event: 'request'; request: NetworkRequest};
  response: {event: 'response'; response: NetworkResponse};
  requestFinished: {event: 'requestFinished'; requestId: number; bytes: number};
  requestFailed: {event: 'requestFailed'; requestId: number; message: string; cancelled: boolean};
  webSocket: {event: 'webSocket'; id: number; url: string; kind: string; data: Buffer; message: string | null};
  eventSource: {event: 'eventSource'; requestId: number; eventName: string; eventId: string; data: string};
}
export type RouteStage = 'request' | 'response' | 'auth';
/** One intercepted request. Complete it once; callbacks cannot reenter their page. */
export class Route {
  private constructor();
  readonly stage: RouteStage;
  readonly request: NetworkRequest;
  readonly response: NetworkResponse | null;
  readonly authChallenge: string | null;
  readonly pending: boolean;
  body(): Buffer | null;
  continueRequest(overrides?: {url?: string; method?: string; headers?: Headers; body?: string | Uint8Array; status?: number}): void;
  fulfill(options?: {status?: number; headers?: Headers; body?: string | Uint8Array}): void;
  fail(message?: string): void;
  authenticate(username: string, password: string): void;
}
export interface Page {
  on<E extends keyof PageEvents>(event: E, handler: (record: PageEvents[E]) => void | Promise<void>): () => void;
  route(pattern: string, handler: (route: Route) => void | Promise<void>, options?: {stage?: RouteStage}): () => void;
  configureNetwork(options?: NetworkOptions): Promise<void>;
  setExtraHeaders(headers: Headers): void;
  clearCache(): void;
  responseBody(requestId: number): Buffer;
}

export function graphicsPlugin(options?: {rgba?: number[]; renderer?: string; webgpu?: boolean}, handlers?: PluginDescriptor['handlers']): Plugin;
export function audioPlugin(options?: {sampleRate?: number}, handlers?: PluginDescriptor['handlers']): Plugin;
export function rtcPlugin(): Plugin;

export interface PluginInfo {
  defaultEnabled: boolean;
  installers: {node: string; python: string};
  capabilities: string[];
  limitations: string[];
  lifetime: string;
  options: Record<string, {default: JSONValue; python?: string; hostOnly?: boolean}>;
}
export function browserPlugins(): Plugin[];
export function pluginCatalog(): Record<string, PluginInfo>;
export type StorageOperation = {type: 'get' | 'delete'; key: string} | {type: 'set'; key: string; value: HostValue} | {type: 'clear'};
export interface HostStorageBackend {
  transaction(origin: string, operations: StorageOperation[]): HostValue[] | Promise<HostValue[]>;
  close?(): void | Promise<void>;
}
export function hostStoragePlugin(options?: {backendFactory?: () => HostStorageBackend}): Plugin;
