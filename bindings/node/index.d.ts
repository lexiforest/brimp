export interface SessionOptions {
  personaJson?: string
  caBundle?: string
  enableWorker?: boolean
  enableStreamingNetworking?: boolean
  enableCanvas?: boolean
  enableWebGL?: boolean
  enableWebGPU?: boolean
  enableWebAudio?: boolean
  enableWebAudioOutput?: boolean
  storagePath?: string
  storageQuotaBytes?: number
  headers?: Record<string, string | number | boolean>
  params?: URLSearchParams | Record<string, QueryValue>
  cookies?: Record<string, string | number | boolean>
  auth?: readonly [string, string]
  proxy?: string
  referer?: string
  timeoutMs?: number
  allowRedirects?: boolean
  maxRedirects?: number
  defaultEncoding?: string
}

export type QueryValue = string | number | boolean | readonly (string | number | boolean)[]

export interface PageOptions { proxy?: string }

export interface RequestOptions {
  params?: URLSearchParams | Record<string, QueryValue>
  data?: string | Buffer | URLSearchParams | Record<string, string | number | boolean>
  content?: string | Buffer | Uint8Array
  json?: unknown
  multipart?: Multipart
  headers?: Record<string, string | number | boolean>
  cookies?: Record<string, string | number | boolean>
  auth?: readonly [string, string]
  timeoutMs?: number
  allowRedirects?: boolean
  maxRedirects?: number
  proxy?: string
  referer?: string
  signal?: AbortSignal
}

export interface ScreenshotOptions { path?: string | Buffer | URL; fullPage?: boolean }
export interface ExtractionOptions { contentSelector?: string; removeImages?: boolean; language?: string; debug?: boolean }

export interface ExtractedDocument {
  title: string
  description: string
  domain: string
  favicon: string
  image: string
  language: string
  parseTime: number
  published: string
  author: string
  site: string
  schemaOrgData: unknown
  wordCount: number
  content: string
  contentMarkdown?: string
  extractorType?: string
  metaTags?: Array<{ name?: string; property?: string; content?: string }>
  debug?: unknown
  profile?: Record<string, number>
  variables?: Record<string, string>
}

export declare class RequestError extends Error { readonly code: string }
export declare class ConnectionError extends RequestError {}
export declare class Timeout extends RequestError {}
export declare class TooManyRedirects extends ConnectionError {}
export declare class InvalidRequest extends RequestError {}
export declare class InvalidURL extends InvalidRequest {}
export declare class JavaScriptError extends RequestError {}
export declare class SessionClosed extends RequestError {}
export declare class HTTPError extends RequestError { readonly page: Page }

export declare class Headers implements Iterable<[string, string]> {
  get(name: string): string | undefined
  getAll(name: string): string[]
  has(name: string): boolean
  entries(): Iterator<[string, string]>
  keys(): Iterator<string>
  values(): Iterator<string | undefined>
  readonly raw: string[][]
  [Symbol.iterator](): Iterator<[string, string]>
}

export interface SentRequest {
  readonly method: string
  readonly url: string
  readonly headers: Headers
  readonly body?: Buffer
}

export interface Redirect {
  readonly statusCode: number
  readonly reason: string
  readonly url: string
  readonly headers: Headers
  readonly request: SentRequest
}

export declare class Multipart {
  addPart(options: {
    name: string
    data?: string | Buffer | Uint8Array
    localPath?: string | Buffer | URL
    filename?: string
    contentType?: string
  }): this
}

export declare class Page {
  readonly statusCode: number
  readonly reason: string
  readonly url: string
  readonly headers: Headers
  readonly content: Buffer
  readonly html: string | null
  readonly cookies: Record<string, string>
  readonly elapsed: number
  readonly ok: boolean
  encoding: string
  readonly text: string
  readonly lastRequest: SentRequest
  readonly history: readonly Redirect[]
  readonly redirectCount: number
  readonly httpVersion: string | null
  readonly downloadedBytes: number
  readonly uploadedBytes: number
  readonly headerBytes: number
  request(method: string, url: string | URL, options?: RequestOptions): Promise<this>
  get(url: string | URL, options?: RequestOptions): Promise<this>
  head(url: string | URL, options?: RequestOptions): Promise<this>
  options(url: string | URL, options?: RequestOptions): Promise<this>
  delete(url: string | URL, options?: RequestOptions): Promise<this>
  post(url: string | URL, options?: RequestOptions): Promise<this>
  put(url: string | URL, options?: RequestOptions): Promise<this>
  patch(url: string | URL, options?: RequestOptions): Promise<this>
  json(): unknown
  raiseForStatus(): void
  evaluate(expression: string): Promise<unknown>
  screenshot(options?: ScreenshotOptions): Promise<Buffer>
  extract(options?: ExtractionOptions): Promise<ExtractedDocument>
  click(selector: string): Promise<void>
  hover(selector: string): Promise<void>
  type(selector: string, text: string): Promise<void>
  tap(selector: string): Promise<void>
  close(): Promise<void>
}

export declare class Session {
  headers: Record<string, string | number | boolean>
  params?: URLSearchParams | Record<string, QueryValue>
  cookies: Record<string, string | number | boolean>
  auth?: readonly [string, string]
  proxy?: string
  referer?: string
  timeoutMs: number
  allowRedirects: boolean
  maxRedirects: number
  defaultEncoding: string
  newPage(options?: PageOptions): Promise<Page>
  request(method: string, url: string | URL, options?: RequestOptions): Promise<Page>
  get(url: string | URL, options?: RequestOptions): Promise<Page>
  head(url: string | URL, options?: RequestOptions): Promise<Page>
  options(url: string | URL, options?: RequestOptions): Promise<Page>
  delete(url: string | URL, options?: RequestOptions): Promise<Page>
  post(url: string | URL, options?: RequestOptions): Promise<Page>
  put(url: string | URL, options?: RequestOptions): Promise<Page>
  patch(url: string | URL, options?: RequestOptions): Promise<Page>
  close(): Promise<void>
}

export declare function createSession(options?: SessionOptions): Promise<Session>
export declare function request(method: string, url: string | URL, options?: SessionOptions & RequestOptions): Promise<Page>
export declare function get(url: string | URL, options?: SessionOptions & RequestOptions): Promise<Page>
export declare function head(url: string | URL, options?: SessionOptions & RequestOptions): Promise<Page>
export declare function options(url: string | URL, options?: SessionOptions & RequestOptions): Promise<Page>
export declare function delete_(url: string | URL, options?: SessionOptions & RequestOptions): Promise<Page>
export { delete_ as delete }
export declare function post(url: string | URL, options?: SessionOptions & RequestOptions): Promise<Page>
export declare function put(url: string | URL, options?: SessionOptions & RequestOptions): Promise<Page>
export declare function patch(url: string | URL, options?: SessionOptions & RequestOptions): Promise<Page>
