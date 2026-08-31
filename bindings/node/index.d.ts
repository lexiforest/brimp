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
}

export type QueryValue = string | number | boolean | readonly (string | number | boolean)[]

export interface PageOptions {
  proxy?: string
}

export interface GetOptions {
  params?: URLSearchParams | Record<string, QueryValue>
  headers?: Record<string, string | number | boolean>
  cookies?: Record<string, string | number | boolean>
  timeoutMs?: number
  signal?: AbortSignal
}

export interface ScreenshotOptions {
  path?: string | Buffer | URL
  fullPage?: boolean
}

export interface ExtractionOptions {
  contentSelector?: string
  removeImages?: boolean
  language?: string
  debug?: boolean
}

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

export declare class BrimpError extends Error { readonly code: string }
export declare class ConnectionError extends BrimpError {}
export declare class Timeout extends BrimpError {}
export declare class TooManyRedirects extends ConnectionError {}
export declare class InvalidRequest extends BrimpError {}
export declare class InvalidURL extends InvalidRequest {}
export declare class JavaScriptError extends BrimpError {}
export declare class HTTPError extends BrimpError { readonly response: Response }

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

export declare class Response {
  readonly statusCode: number
  readonly reason: string
  readonly url: string
  readonly headers: Headers
  readonly content: Buffer
  readonly html: string | null
  readonly cookies: Record<string, string>
  readonly elapsed: number
  readonly ok: boolean
  readonly encoding: string
  readonly text: string
  json(): unknown
  raiseForStatus(): void
}

export declare class Page {
  get(url: string | URL, options?: GetOptions): Promise<Response>
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
  cookies: Record<string, string | number | boolean>
  newPage(options?: PageOptions): Promise<Page>
  close(): Promise<void>
}

export declare function createSession(options?: SessionOptions): Promise<Session>
export declare function get(url: string | URL, options?: SessionOptions & PageOptions & GetOptions): Promise<Response>
