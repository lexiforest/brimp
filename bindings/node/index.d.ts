export interface LaunchOptions { personaJson?: string }
export interface GotoOptions { timeoutMs?: number; signal?: AbortSignal }
export interface ScreenshotOptions { fullPage?: boolean }
export declare class BrimpError extends Error {
  readonly code: 'invalid_input' | 'transport' | 'http_status' | 'navigation' | 'javascript' | 'timeout' | 'cancelled' | 'unsupported' | 'closed' | 'screenshot' | 'internal'
}
export declare class Page {
  goto(url: string, options?: GotoOptions): Promise<void>
  evaluate(expression: string): Promise<unknown>
  title(): Promise<string>
  textContent(): Promise<string>
  screenshot(options?: ScreenshotOptions): Promise<Buffer>
  close(): Promise<void>
}
export declare class Browser {
  newPage(): Promise<Page>
  close(): Promise<void>
}
export declare function launch(options?: LaunchOptions): Promise<Browser>
