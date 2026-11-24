"""Run with BDHTML2PDF_BINARY pointing to a built bdhtml2pdf:

    PYTHONPATH=src python3 -m unittest discover -s tests
"""

import os
import re
import subprocess
import tempfile
import unittest

import bdhtml2pdf

PAGE = '<!DOCTYPE html><p id="p"></p><script>p.textContent = "Merhaba İğdır"</script>'


def start_server(*extra):
    p = subprocess.Popen(
        [bdhtml2pdf._binary(None), "serve", "--listen", "127.0.0.1:0", "--jobs", "2", *extra],
        stderr=subprocess.PIPE,
        text=True,
    )
    line = p.stderr.readline()
    url = re.search(r"(http://\S+)", line).group(1)
    return p, url


class LocalTest(unittest.TestCase):
    def test_pdf(self):
        pdf = bdhtml2pdf.convert(PAGE, page_size="A5", title="Fatura")
        self.assertTrue(pdf.startswith(b"%PDF-"))
        self.assertIn(b"419.52", pdf)

    def test_html_shows_that_scripts_ran(self):
        self.assertIn('<p id="p">Merhaba İğdır', bdhtml2pdf.convert(PAGE, "html").decode())

    def test_png(self):
        self.assertTrue(bdhtml2pdf.convert(PAGE, "png", width=300, height=200).startswith(b"\x89PNG"))

    def test_pdf_a(self):
        pdf = bdhtml2pdf.convert("<title>T</title><p>x</p>", pdf_a="2b")
        self.assertIn(b"<pdfaid:part>2</pdfaid:part>", pdf)

    def test_false_and_none_leave_a_flag_out(self):
        self.assertEqual(
            bdhtml2pdf.flags({"disable_javascript": True, "quiet": False, "title": None, "zoom": 2}),
            ["--disable-javascript", "--zoom=2"],
        )

    def test_relative_files_load_from_base_dir(self):
        with tempfile.TemporaryDirectory() as d:
            with open(os.path.join(d, "s.js"), "w") as f:
                f.write('document.body.textContent = "from s.js"')
            html = bdhtml2pdf.convert('<body><script src="s.js"></script>', "html", base_dir=d)
            self.assertIn("from s.js", html.decode())

    def test_errors_carry_the_message(self):
        with self.assertRaises(bdhtml2pdf.Bdhtml2pdfError) as e:
            bdhtml2pdf.convert("<p>x</p>", page_size="Z9")
        self.assertIn("unknown page size", str(e.exception))
        with self.assertRaises(ValueError):
            bdhtml2pdf.convert("<p>x</p>", "docx")


class ServerTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.proc, url = start_server("--max-memory", "200")
        cls.client = bdhtml2pdf.Client(url)

    @classmethod
    def tearDownClass(cls):
        cls.proc.kill()
        cls.proc.wait()
        cls.proc.stderr.close()

    def test_health(self):
        self.assertTrue(self.client.health().startswith("ok bdhtml2pdf"))

    def test_pdf(self):
        pdf = self.client.convert(PAGE, page_size="A5")
        self.assertTrue(pdf.startswith(b"%PDF-"))
        self.assertIn(b"419.52", pdf)

    def test_html_and_bool_options(self):
        self.assertIn('<p id="p">Merhaba İğdır', self.client.convert(PAGE, "html").decode())
        self.assertIn('<p id="p"></p>', self.client.convert(PAGE, "html", disable_javascript=True).decode())

    def test_errors_carry_the_status(self):
        with self.assertRaises(bdhtml2pdf.Bdhtml2pdfError) as e:
            self.client.convert("<p>x</p>", max_memory=0)
        self.assertEqual(e.exception.status, 400)
        with self.assertRaises(bdhtml2pdf.Bdhtml2pdfError) as e:
            self.client.convert('<script>let s = "x"; for (;;) s = s + s;</script>')
        self.assertEqual(e.exception.status, 507)


if __name__ == "__main__":
    unittest.main()
