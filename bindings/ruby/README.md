# bdhtml2pdf for Ruby

HTML + JavaScript to PDF with [bdhtml2pdf](https://github.com/bahadirdogru/bdhtml2pdf).
No native extension: it runs the `bdhtml2pdf` program, or talks to
`bdhtml2pdf serve`. Install bdhtml2pdf from its releases first.

```ruby
require "bdhtml2pdf"

pdf = Bdhtml2pdf.convert("<h1>Merhaba</h1>", page_size: "A4", pdf_a: "2b")
File.binwrite("out.pdf", pdf)
png = Bdhtml2pdf.convert(html, format: :png, base_dir: __dir__, width: 800)

client = Bdhtml2pdf::Client.new("http://127.0.0.1:7368") # bdhtml2pdf serve
pdf = client.convert("<h1>Merhaba</h1>", margin_top: "20mm")
```

In Rails, `send_data pdf, type: "application/pdf"`. Errors raise
`Bdhtml2pdf::Error` with the message and `#status`. See
[the bindings overview](https://github.com/bahadirdogru/bdhtml2pdf/tree/main/bindings).
