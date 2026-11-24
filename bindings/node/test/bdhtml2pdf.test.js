// Run with BDHTML2PDF_BINARY pointing to a built bdhtml2pdf: npm test
'use strict';
const test = require('node:test');
const assert = require('node:assert');
const { spawn } = require('node:child_process');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { convert, Client, Bdhtml2pdfError, flags } = require('..');

const PAGE = '<!DOCTYPE html><p id="p"></p><script>p.textContent = "Merhaba İğdır"</script>';
const BIN = process.env.BDHTML2PDF_BINARY || 'bdhtml2pdf';

test('pdf with options', async () => {
  const pdf = await convert(PAGE, { pageSize: 'A5', title: 'Fatura' });
  assert.ok(pdf.subarray(0, 5).equals(Buffer.from('%PDF-')));
  assert.ok(pdf.includes('419.52'));
});

test('html shows that scripts ran; png', async () => {
  assert.ok((await convert(PAGE, { format: 'html' })).toString().includes('<p id="p">Merhaba İğdır'));
  const png = await convert(PAGE, { format: 'png', width: 300, height: 200 });
  assert.equal(png[0], 0x89);
});

test('pdf/a', async () => {
  const pdf = await convert('<title>T</title><p>x</p>', { pdf_a: '2b' });
  assert.ok(pdf.includes('<pdfaid:part>2</pdfaid:part>'));
});

test('flags', () => {
  assert.deepEqual(flags({ disableJavascript: true, quiet: false, title: null, zoom: 2, page_size: 'A5' }), [
    '--disable-javascript',
    '--zoom=2',
    '--page-size=A5',
  ]);
});

test('relative files load from baseDir', async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bdhtml2pdf-node-'));
  fs.writeFileSync(path.join(dir, 's.js'), 'document.body.textContent = "from s.js"');
  const html = await convert('<body><script src="s.js"></script>', { format: 'html', baseDir: dir });
  assert.ok(html.toString().includes('from s.js'));
  fs.rmSync(dir, { recursive: true });
});

test('errors carry the message', async () => {
  await assert.rejects(convert('<p>x</p>', { pageSize: 'Z9' }), (e) => e instanceof Bdhtml2pdfError && /unknown page size/.test(e.message));
  assert.throws(() => convert('<p>x</p>', { format: 'docx' }), TypeError);
});

test('server', async (t) => {
  const child = spawn(BIN, ['serve', '--listen', '127.0.0.1:0', '--jobs', '2', '--max-memory', '200']);
  t.after(() => child.kill());
  const url = await new Promise((resolve) => {
    child.stderr.once('data', (d) => resolve(/(http:\/\/\S+)/.exec(d.toString())[1]));
  });
  const client = new Client(url);
  assert.match(await client.health(), /^ok bdhtml2pdf/);
  const pdf = await client.convert(PAGE, { pageSize: 'A5' });
  assert.ok(pdf.includes('419.52'));
  assert.ok((await client.convert(PAGE, { format: 'html', disableJavascript: true })).toString().includes('<p id="p"></p>'));
  await assert.rejects(client.convert('<p>x</p>', { maxMemory: 0 }), (e) => e.status === 400);
  await assert.rejects(client.convert('<script>let s = "x"; for (;;) s = s + s;</script>'), (e) => e.status === 507);
});
