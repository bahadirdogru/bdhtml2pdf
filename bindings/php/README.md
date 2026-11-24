# bdhtml2pdf for PHP

HTML + JavaScript to PDF with [bdhtml2pdf](https://github.com/bahadirdogru/bdhtml2pdf).
No extension: it runs the `bdhtml2pdf` program (`proc_open`), or talks to
`bdhtml2pdf serve` (`allow_url_fopen`). Install bdhtml2pdf from its releases
first.

```php
use Bdhtml2pdf\Converter;
use Bdhtml2pdf\Client;

$pdf = (new Converter())->convert('<h1>Merhaba</h1>', ['page_size' => 'A4', 'pdf_a' => '2b']);
$png = (new Converter(baseDir: __DIR__))->convert($html, ['width' => 800], 'png');

$client = new Client('http://127.0.0.1:7368'); // bdhtml2pdf serve
$pdf = $client->convert('<h1>Merhaba</h1>', ['margin_top' => '20mm']);
```

Errors throw `Bdhtml2pdf\Bdhtml2pdfException` with the message; the code is
the exit status or HTTP status. See
[the bindings overview](https://github.com/bahadirdogru/bdhtml2pdf/tree/main/bindings).
