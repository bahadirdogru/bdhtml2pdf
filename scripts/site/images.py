"""Resize the rendered pictures for the website and refresh the PDF sizes
shown on it. Called by scripts/site.sh as:

    images.py <previews dir> <assets dir> <index.html>
"""

import os
import re
import sys

from PIL import Image

previews, assets, index = sys.argv[1:4]
docs = os.path.dirname(index)


def save(img, stem, webp_only=False):
    """Write `stem`.webp, and `stem`.png as the fallback."""
    img.save(os.path.join(assets, stem + ".webp"), "WEBP", quality=88, method=6)
    if not webp_only:
        img.save(os.path.join(assets, stem + ".png"), "PNG", optimize=True)


def width(img, w):
    return img.resize((w, round(img.height * w / img.width)), Image.LANCZOS)


# The hero: bdhtml2pdf's own image output of the showcase.
hero = Image.open(os.path.join(docs, "showcase", "showcase.png")).convert("RGB")
save(width(hero, 800), "showcase-800")
save(width(hero, 1400), "showcase-1400", webp_only=True)

# Sample previews: the top of a portrait page, or a whole landscape page.
for name in ["report", "invoice", "newsletter", "certificate"]:
    page = width(Image.open(os.path.join(previews, name + ".png")).convert("RGB"), 600)
    if page.height > page.width:
        page = page.crop((0, 0, 600, 780))
    save(page, "sample-" + name)
    print(f"sample-{name}: {page.width}x{page.height}")

for icon in ["og.png", "apple-touch-icon.png"]:
    path = os.path.join(assets, icon)
    Image.open(path).convert("RGB").save(path, "PNG", optimize=True)

# PDF sizes next to the download links.
html = open(index, encoding="utf-8").read()


def size(match):
    href = match.group(1)
    kb = round(os.path.getsize(os.path.join(docs, href)) / 1024)
    return f'<a href="{href}">PDF <span class="meta">{kb} KB</span>'


html = re.sub(r'<a href="([^"]+\.pdf)">PDF <span class="meta">[^<]*</span>', size, html)
open(index, "w", encoding="utf-8").write(html)
