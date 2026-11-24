# bdhtml2pdf for Node.js

HTML + JavaScript to PDF with [bdhtml2pdf](https://github.com/bahadirdogru/bdhtml2pdf).
No native addon: it runs the `bdhtml2pdf` program, or talks to
`bdhtml2pdf serve`. Install bdhtml2pdf from its releases first.

```js
const { convert, Client } = require('bdhtml2pdf');

const pdf = await convert('<h1>Merhaba</h1>', { pageSize: 'A4', pdfA: '2b' });
const png = await convert(html, { format: 'png', baseDir: __dirname, width: 800 });

const client = new Client('http://127.0.0.1:7368'); // bdhtml2pdf serve
const pdf2 = await client.convert('<h1>Merhaba</h1>', { marginTop: '20mm' });
```

Errors reject with `Bdhtml2pdfError` (message and `.status`). TypeScript
types are included. See [the bindings overview](https://github.com/bahadirdogru/bdhtml2pdf/tree/main/bindings).
