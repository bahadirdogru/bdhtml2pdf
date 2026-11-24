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
LANGS = ["arabic", "persian", "japanese", "chinese", "korean", "russian"]
for name in ["report", "invoice", "newsletter", "ticket", "certificate"] + LANGS:
    page = width(Image.open(os.path.join(previews, name + ".png")).convert("RGB"), 600)
    if page.height > page.width:
        page = page.crop((0, 0, 600, 780))
    save(page, "sample-" + name)
    print(f"sample-{name}: {page.width}x{page.height}")

# Chromium vs bdhtml2pdf: whole first pages, the same size, WebP only.
chromium = os.path.join(previews, "chromium")
if os.path.isdir(chromium):
    for name in LANGS:
        for engine, path in [("chromium", os.path.join(chromium, name + ".png")),
                             ("bdhtml2pdf", os.path.join(previews, name + ".png"))]:
            page = Image.open(path).convert("RGB").resize((560, 792), Image.LANCZOS)
            page.save(os.path.join(assets, f"compare-{name}-{engine}.webp"),
                      "WEBP", quality=88, method=6)
        print(f"compare-{name}: 560x792")

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

# The Chromium version the comparison was made with.
version = os.path.join(previews, "chromium-version")
if os.path.exists(version):
    v = open(version).read().strip()
    html = re.sub(r'<span class="chromium-version">[^<]*</span>',
                  f'<span class="chromium-version">{v}</span>', html)
open(index, "w", encoding="utf-8").write(html)
