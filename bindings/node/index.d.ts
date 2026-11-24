/// <reference types="node" />

export type Format = 'pdf' | 'png' | 'jpg' | 'jpeg' | 'html';

/** Command-line flags in camelCase, e.g. pageSize, marginTop, pdfA, disableJavascript. */
export interface Options {
  format?: Format;
  pageSize?: string;
  orientation?: 'Portrait' | 'Landscape';
  pageWidth?: string;
  pageHeight?: string;
  marginTop?: string;
  marginBottom?: string;
  marginLeft?: string;
  marginRight?: string;
  title?: string;
  pdfA?: '1b' | '2b' | '2u' | '2a' | '3b' | '3u' | '3a';
  enableSmartShrinking?: boolean;
  disableSmartShrinking?: boolean;
  width?: number;
  height?: number;
  zoom?: number;
  quality?: number;
  disableJavascript?: boolean;
  javascriptDelay?: number;
  windowStatus?: string;
  javascriptLoopLimit?: number;
  [flag: string]: string | number | boolean | null | undefined;
}

export interface LocalOptions extends Options {
  /** Where relative images, styles and scripts load from. */
  baseDir?: string;
  /** Path to bdhtml2pdf; default $BDHTML2PDF_BINARY, then PATH. */
  binary?: string;
  /** Milliseconds before the process is stopped. */
  timeout?: number;
}

export function convert(html: string | Buffer, options?: LocalOptions): Promise<Buffer>;
export function flags(options?: Options): string[];

export class Client {
  constructor(url?: string);
  url: string;
  health(): Promise<string>;
  convert(html: string | Buffer, options?: Options): Promise<Buffer>;
}

export class Bdhtml2pdfError extends Error {
  /** Exit status, or HTTP status from a server. */
  status?: number;
}
