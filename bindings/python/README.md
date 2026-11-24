# bdhtml2pdf for Python

HTML + JavaScript to PDF with [bdhtml2pdf](https://github.com/bahadirdogru/bdhtml2pdf).
No native extension: it runs the `bdhtml2pdf` program, or talks to
`bdhtml2pdf serve`. Install bdhtml2pdf from its releases first.

```python
import bdhtml2pdf

pdf = bdhtml2pdf.convert("<h1>Merhaba</h1>", page_size="A4", pdf_a="2b")
png = bdhtml2pdf.convert(open("chart.html").read(), "png", base_dir=".", width=800)

client = bdhtml2pdf.Client("http://127.0.0.1:7368")   # bdhtml2pdf serve
pdf = client.convert("<h1>Merhaba</h1>", margin_top="20mm")
```

Errors raise `bdhtml2pdf.Bdhtml2pdfError` with the message and `.status`.
See [the bindings overview](https://github.com/bahadirdogru/bdhtml2pdf/tree/main/bindings).
