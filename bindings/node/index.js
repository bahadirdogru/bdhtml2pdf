'use strict';
// HTML + JavaScript to PDF with bdhtml2pdf.
//
// convert() runs the bdhtml2pdf program, one process per page;
// new Client(url).convert() sends the page to a `bdhtml2pdf serve` server.
// Options are the command-line flags in camelCase or snake_case:
// { pageSize: 'A5', marginTop: '20mm', pdfA: '2b', disableJavascript: true }.

const { spawn } = require('node:child_process');
const fs = require('node:fs');
const path = require('node:path');

const FORMATS = ['pdf', 'png', 'jpg', 'html'];

class Bdhtml2pdfError extends Error {
  /** @param {string} message @param {number|undefined} status exit status or HTTP status */
  constructor(message, status) {
    super(message);
    this.name = 'Bdhtml2pdfError';
    this.status = status;
  }
}

/** pageSize, page_size → page-size */
function flagName(key) {
  return key.replace(/_/g, '-').replace(/[A-Z]/g, (c) => '-' + c.toLowerCase());
}

/** Options as command-line flags; true is a bare flag, false/null/undefined leave it out. */
function flags(options = {}) {
  const out = [];
  for (const [key, value] of Object.entries(options)) {
    if (value === undefined || value === null || value === false) continue;
    out.push(value === true ? `--${flagName(key)}` : `--${flagName(key)}=${value}`);
  }
  return out;
}

function checkFormat(format) {
  let f = String(format).toLowerCase();
  if (f === 'jpeg') f = 'jpg';
  if (!FORMATS.includes(f)) throw new TypeError(`unknown format ${JSON.stringify(format)}; expected one of ${FORMATS.join(', ')}`);
  return f;
}

function findBinary(binary) {
  if (binary) return binary;
  if (process.env.BDHTML2PDF_BINARY) return process.env.BDHTML2PDF_BINARY;
  const exe = process.platform === 'win32' ? 'bdhtml2pdf.exe' : 'bdhtml2pdf';
  for (const dir of (process.env.PATH || '').split(path.delimiter)) {
    const p = path.join(dir, exe);
    if (dir && fs.existsSync(p)) return p;
  }
  throw new Bdhtml2pdfError('bdhtml2pdf not found; install it, put it on PATH or set BDHTML2PDF_BINARY');
}

function message(buf, fallback) {
  return (buf.toString('utf8').trim() || fallback).replace(/^error: /, '');
}

/**
 * Convert html and resolve to the PDF (or PNG, JPEG, HTML) bytes.
 * @param {string|Buffer} html
 * @param {object} [options] flags, plus format, baseDir (where relative files load from),
 *   binary (path to bdhtml2pdf) and timeout (ms)
 * @returns {Promise<Buffer>}
 */
function convert(html, options = {}) {
  const { format = 'pdf', baseDir, binary, timeout, ...rest } = options;
  const args = ['-q', `--format=${checkFormat(format)}`, ...flags(rest), '-', '-'];
  return new Promise((resolve, reject) => {
    const child = spawn(findBinary(binary), args, { cwd: baseDir, timeout, windowsHide: true });
    const out = [];
    const err = [];
    child.stdout.on('data', (d) => out.push(d));
    child.stderr.on('data', (d) => err.push(d));
    child.on('error', reject);
    child.on('close', (code, signal) => {
      if (code === 0) return resolve(Buffer.concat(out));
      const fallback = signal ? `stopped by ${signal}` : `exit status ${code}`;
      reject(new Bdhtml2pdfError(message(Buffer.concat(err), fallback), code ?? undefined));
    });
    child.stdin.on('error', () => {}); // the child may exit before reading everything
    child.stdin.end(html);
  });
}

/**
 * A client for `bdhtml2pdf serve`. The server loads nothing from its disk, so a page
 * carries its images and styles inline (data: URLs, <style>).
 */
class Client {
  constructor(url = 'http://127.0.0.1:7368') {
    this.url = url.replace(/\/+$/, '');
  }

  async health() {
    const r = await fetch(`${this.url}/health`);
    return (await r.text()).trim();
  }

  /** @returns {Promise<Buffer>} */
  async convert(html, options = {}) {
    const { format = 'pdf', ...rest } = options;
    const query = new URLSearchParams();
    for (const [key, value] of Object.entries(rest)) {
      if (value === undefined || value === null || value === false) continue;
      query.set(flagName(key), value === true ? 'true' : String(value));
    }
    const qs = query.toString();
    const r = await fetch(`${this.url}/${checkFormat(format)}${qs ? '?' + qs : ''}`, {
      method: 'POST',
      headers: { 'Content-Type': 'text/html; charset=utf-8' },
      body: html,
    });
    const body = Buffer.from(await r.arrayBuffer());
    if (!r.ok) throw new Bdhtml2pdfError(message(body, `HTTP ${r.status}`), r.status);
    return body;
  }
}

module.exports = { convert, Client, Bdhtml2pdfError, flags };
