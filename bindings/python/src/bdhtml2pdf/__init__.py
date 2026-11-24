"""HTML + JavaScript to PDF with bdhtml2pdf.

Two ways to convert, with the same options:

- ``convert()`` runs the ``bdhtml2pdf`` program, one process per page.
- ``Client(url).convert()`` sends the page to a ``bdhtml2pdf serve`` server.

Options are the command-line flags in snake_case: ``page_size="A5"``,
``margin_top="20mm"``, ``pdf_a="2b"``, ``disable_javascript=True``.
"""

from __future__ import annotations

import os
import shutil
import subprocess
import urllib.error
import urllib.parse
import urllib.request
from typing import Optional, Union

__all__ = ["Bdhtml2pdfError", "Client", "convert", "flags"]
__version__ = "0.1.3"

FORMATS = ("pdf", "png", "jpg", "html")


class Bdhtml2pdfError(Exception):
    """A conversion failed. ``status`` is the exit status or HTTP status."""

    def __init__(self, message: str, status: Optional[int] = None):
        super().__init__(message)
        self.status = status


def _name(key: str) -> str:
    return key.replace("_", "-")


def flags(options: dict) -> list:
    """Options as command-line flags. ``True`` is a bare flag; ``False`` and
    ``None`` leave the flag out. Values use ``--flag=value``, so a value that
    starts with a dash stays a value."""
    out = []
    for key, value in options.items():
        if value is None or value is False:
            continue
        if value is True:
            out.append(f"--{_name(key)}")
        else:
            out.append(f"--{_name(key)}={value}")
    return out


def _format(format: str) -> str:
    format = format.lower()
    if format == "jpeg":
        format = "jpg"
    if format not in FORMATS:
        raise ValueError(f"unknown format {format!r}; expected one of {', '.join(FORMATS)}")
    return format


def _binary(binary: Optional[str]) -> str:
    found = binary or os.environ.get("BDHTML2PDF_BINARY") or shutil.which("bdhtml2pdf")
    if not found:
        raise Bdhtml2pdfError(
            "bdhtml2pdf not found; install it, put it on PATH or set BDHTML2PDF_BINARY"
        )
    return found


def convert(
    html: Union[str, bytes],
    format: str = "pdf",
    *,
    base_dir: Optional[str] = None,
    binary: Optional[str] = None,
    timeout: Optional[float] = None,
    **options,
) -> bytes:
    """Convert ``html`` and return the PDF (or PNG, JPEG, HTML) bytes.

    ``base_dir`` is where relative images, styles and scripts are loaded
    from; it defaults to the current directory. ``binary`` is the path to
    ``bdhtml2pdf`` (default: ``$BDHTML2PDF_BINARY``, then ``PATH``)."""
    if isinstance(html, str):
        html = html.encode("utf-8")
    cmd = [_binary(binary), "-q", f"--format={_format(format)}", *flags(options), "-", "-"]
    try:
        p = subprocess.run(
            cmd, input=html, capture_output=True, cwd=base_dir, timeout=timeout, check=False
        )
    except subprocess.TimeoutExpired as e:
        raise Bdhtml2pdfError(f"conversion took longer than {timeout} s") from e
    if p.returncode != 0:
        message = p.stderr.decode("utf-8", "replace").strip() or f"exit status {p.returncode}"
        raise Bdhtml2pdfError(message.removeprefix("error: "), p.returncode)
    return p.stdout


class Client:
    """A client for ``bdhtml2pdf serve``.

    The server loads nothing from its disk, so a page must carry its images
    and styles inline (``data:`` URLs, ``<style>``) or the server must be
    able to reach them."""

    def __init__(self, url: str = "http://127.0.0.1:7368", timeout: float = 60):
        self.url = url.rstrip("/")
        self.timeout = timeout

    def health(self) -> str:
        with urllib.request.urlopen(f"{self.url}/health", timeout=self.timeout) as r:
            return r.read().decode().strip()

    def convert(self, html: Union[str, bytes], format: str = "pdf", **options) -> bytes:
        if isinstance(html, str):
            html = html.encode("utf-8")
        query = {}
        for key, value in options.items():
            if value is None or value is False:
                continue
            query[_name(key)] = "true" if value is True else str(value)
        url = f"{self.url}/{_format(format)}"
        if query:
            url += "?" + urllib.parse.urlencode(query)
        req = urllib.request.Request(
            url, data=html, method="POST", headers={"Content-Type": "text/html; charset=utf-8"}
        )
        try:
            with urllib.request.urlopen(req, timeout=self.timeout) as r:
                return r.read()
        except urllib.error.HTTPError as e:
            message = e.read().decode("utf-8", "replace").strip()
            raise Bdhtml2pdfError(message.removeprefix("error: "), e.code) from None
