# Next session

Status on 2026-11-22. The fulgur and Blitz changes live on the fork branches
pinned in `.github/upstream.env` (`scripts/upstream.sh pin` after every push
to them).

## Done in the last session

- Language samples on the website: Arabic, Persian, Japanese, Chinese,
  Korean, Russian (`docs/samples/*.html`, rendered by `scripts/site.sh`).
- "Chromium render vs bdhtml2pdf render" section on the website. Built by
  `scripts/site.sh` when Chromium is installed; tabs are CSS only, since the
  site's CSP has `script-src 'none'`.
- `@page` margins and `@page size` win when no CLI flag is given, as in
  Chromium (`tests/margins.rs`, `tests/page_size.rs`).
- Content is clipped to the page area as Chromium prints it: fulgur
  `clip_to_page_area` (35f70017), turned on in `src/pdf.rs`. This fixed the
  hanko of the Japanese sample, which drew into the right margin.
- Blitz: `<br>` after text in a flex item breaks the line. Issue
  DioxusLabs/blitz#1708, PR #1709 (branch `pr/flex-br`), also on
  `public-layout-pass`.
- fulgur: GPOS y offset sign, margin box alignment, RTL margin boxes,
  `default_margin`, border colors per side, 3px double borders (#969 items
  31–38).

## To do

1. **Layout width: fixed and pinned.** Blitz `Viewport::set_logical_size`
   (an exact, fractional CSS size; `window_size` stays the rounded-up
   physical size), used by fulgur `parse_inner` and `set_viewport_size_px`:
   Blitz c13cfe7d on `public-layout-pass`, fulgur 62e68b87 on
   `blitz-master`; test `tests/layout_width.rs`.
   - Page area origin: done (2026-11-22). Chromium rounds the page area's
     top left corner to the nearest whole CSS px of the printed page
     (10.1mm 38.17 -> 38, 10.3mm 38.93 -> 39, 0.2mm 0.76 -> 1) and keeps the
     exact width. fulgur `Config::snap_page_area_origin` (a8baa4b0), turned
     on in `src/pdf.rs`; tests `tests/layout_width.rs` and fulgur
     `tests/page_area_origin.rs`. Red full-width box at 0, 10, 18, 20 and
     25.4mm: left and top edges now the same as Chromium.
   - **Whole-pixel boxes: checked, not a bug (2026-11-23).** The idea was
     that Blitz rounds `final_layout` to whole px and Chromium paints the
     fractional layout. Measured at 960 dpi on probe boxes (61.3%,
     123.45px, 33.33% + 0.4px) with a Blitz `DocumentConfig::round_layout`
     option (branch `pr/round-layout` in ../blitz, not pushed): Chromium
     pixel-snaps box backgrounds in print too. LTR edges (px from the page
     left): Chromium 479 / 161 / 278, bdhtml2pdf rounded 478 / 161 / 278,
     unrounded 478.2 / 161.4 / 277.7. Unrounded output moved away from
     Chromium, broke `tests/tables.rs` (a blended seam pixel) and the
     low-resolution QR test, so it was dropped; Blitz keeps rounding.
   - **Right-to-left page area: a Chromium quirk, left as it is
     (2026-11-23).** Probe with whole-pixel boxes (100%, 100px, 300px;
     `@page { size: A4; margin: 10mm }`, edges in px from the page left):
     LTR Chromium 38–756.14 / 38–138 / 38–338, here 38–756 / 38–138 /
     38–338. RTL Chromium clips the page area at 36.70–754.84 (shifted 1.3px
     left) but places the boxes from 755.7, so a 100px box shows only
     99.14px (655.70–754.84) and every right edge loses 0.86px; here
     38–756 / 656–756 / 456–756, the boxes whole. Copying that would cut
     boxes, so the Arabic grand-total bar (60.1 vs 60.8) stays as it is.
   - `tests/qrcode.rs` low-resolution test: decoding at exactly 200 dpi
     depends on where module edges fall between pixels (with or without the
     snap, 17–34% of 140–300 dpi fails), so it now asks for 8 of 195–205 dpi.
   - The Korean menu grid columns are now 314px in both engines.
   - Blitz PR for `set_logical_size`: DioxusLabs/blitz#1717 (branch
     `pr/logical-size`, on upstream `main`, with tests in `blitz-tests`).
     Only following it.
2. **"Stone-pot" line break in the Korean menu: fixed (2026-11-23).** With the CJK
   fonts installed (see Rules) and item 1, the column is 314px in both
   engines and "...비빔밥 · Stone-" is 299px wide, so it fits; Chromium
   breaks after the hyphen, bdhtml2pdf does not. ICU4X
   (`icu_segmenter` 2.3.0, `LineBreakWordOption::KeepAll`, used by
   `parley_engine` `analysis.rs`) drops the break after a hyphen even in
   Latin text: `abc · Stone-pot bibimbap` gives `Stone-pot ` as one segment
   with KeepAll, `Stone-` + `pot ` with Normal. CSS Text 3 suppresses only
   breaks between letters (NU, AL, AI, ID classes), so the HY break stays.
   - Reported: unicode-org/icu4x#8772 (also in 2.4.0). **Fixed in the fork
     (2026-11-23):** `HY` was in the keep-all letter-unit lists of the v1
     and v3 segmenters (`components/segmenter/src/line/v{1,3}.rs`); the
     neo segmenter's CLDR tailoring was already right. Branch
     `keep-all-hyphen` of bahadirdogru/icu4x (15e442534, on upstream
     main), tests in `tests/css_word_break.rs`; PR unicode-org/icu4x#8780
     (CLA signed 2026-11-24; waiting for review).
   - **In bdhtml2pdf (2026-11-23):** the tree locks `icu_segmenter` 2.3.0
     (Blitz, Parley, Stylo). Branch `keep-all-hyphen-2.3` of the fork is
     `icu@2.3.0` plus the fix; `cargo package` of it is the orphan branch
     `icu_segmenter-2.3.0-patched` (6a1c97ab, `Cargo.toml` identical to the
     published crate), which `[patch.crates-io]` pins, so cargo fetches
     4 MB, not ICU4X. Test
     `tests/pdf_content.rs::keep_all_breaks_after_a_hyphen_as_normal_does`.
     The Korean menu now breaks `… 비빔밥 · Stone-` / `pot bibimbap`, as
     Chromium does. Drop the patch once a release with #8780 is in the tree.
   - The earlier scratch test (`aaaa Stone-pot bibimbap` at 120px) did not
     show this because both engines broke at the space there.
   - The Japanese hanko: compare again now that the fonts are installed.
3. **fulgur PR #969:** items 39 (`clip_to_page_area`), 40 (exact page area
   width) and 41 (`pdf_a`) are in the PR body.
4. **Japanese sample: fixed (2026-11-24).** The bottom border of the item
   table differed slightly from Chromium. **Measured (2026-11-23, 960 dpi, y in px):** the long
   lines and the item-row separators match within 0.1–0.3 (282.6/282.7,
   346.2/346.3, rows 33.8 / 34.7 / 33.8 / 34.8 in both). The gap is in the
   two borderless sum rows (`.sum td { border: 0; padding: 3px 8px }`,
   小計 and 消費税): 570.7 → 627.3 = 56.6 here vs 55.7 in Chromium, so the
   合計 top border (1.5px ink) is 0.7 lower.
   - **Cause found (probe, 960 dpi):** with `border-collapse`, two rows
     whose cells have `border: 0` get a 1px empty gap between them when
     other cells of the table have a 1px border. A `<tbody>` with one
     bordered row (`td { border: 1px solid }`) and a `<tbody class=sum>` with
     `td { border: 0 }` rows 小計 / 消費税: here 小計 27.0px, a 1.0px
     unpainted gap, 消費税 28.0px; Chromium 27.0 + 28.0 with no gap. The
     collapsed border between the two sum rows must be 0 (both sides 0).
     Text heights are the same in both engines (rows of 28 / 27 px).
   - **Where:** Blitz lays a collapsed table out as a Taffy grid with one
     `gap` per axis (`layout/table.rs`, `style.gap` for
     `BorderCollapse::Collapse`): upstream takes the first cell's border,
     our eee34770 the widest cell border, so every row line gets 1px even
     between borderless rows. Chromium resolves each line on its own.
   - **Rows: fixed in the forks (2026-11-24).** Blitz f6a61067
     (`public-layout-pass`): each horizontal line takes the widest border of
     the cells meeting it (`TableContext::row_lines`), laid out as a
     fixed-size row track between the rows' tracks (row gap 0; cells at
     line `2·row + 1`, span `2·rowspan − 1`; `TableCell::rowspan` public).
     fulgur 2be81aa8 (`blitz-master`): `apply_collapsed` centres each cell's
     top/bottom border on its own lines. Probe now equal to Chromium; the
     Japanese 合計 line is at 626.3 vs Chromium 626.6 (was 627.3; the rest of
     the page is 0.1–0.3 off the same way). Test
     `tests/tables.rs::collapsed_rows_without_borders_get_no_gap_between_them`.
   - **Upstream (2026-11-24):** Blitz `main` still paints a synthesized grid
     from the first cell's border (`draw_table_borders`), so per-line widths
     only make sense there together with per-cell collapsed border
     painting. Commented on DioxusLabs/blitz#504 with the repro and
     f6a61067, offering a PR with both if the approach suits them; fulgur
     PR #969 item 47 for 2be81aa8. Only following #504 now. Columns: same
     idea, later (one gap per axis still applies to vertical lines).
5. **Known browser differences, left as they are** (written in Blitz PR #1709):
   - A `<br>` with no text before it in a flex container: Chromium wraps it
     into the anonymous item, Firefox does not; Blitz follows Firefox.
   - `one<br><br>three` in a flex item: Chromium 60px (Blink ignores
     `display` on `<br>`), Firefox and Blitz 40px.
6. **Inline backgrounds and borders are not drawn** (`<span>` with a
   background or border, `<mark>`); `inline-block` is fine. Also on fulgur
   `main` (bd6222fe). Reported: fulgur-rs/fulgur#973.
   - **Backgrounds: fixed in the fork (2026-11-24).** fulgur d6bc3673
     (`blitz-master`, PR #969 item 48): each glyph run carries the
     background colors of the inline elements it is in, drawn per element
     and line over the fonts' ascent/descent behind the glyphs, as an
     artifact. Issue example at 96 dpi: 2378 yellow pixels here, 2420 in
     Chromium. `<mark>` uses Stylo's `Mark` color rgb(255,235,59) (Firefox's;
     Chromium's is pure yellow). Tests: fulgur `inline_background.rs`,
     `tests/pdf_content.rs::inline_backgrounds_are_drawn`.
   - **Borders and padding: a layout feature, not started (2026-11-24).**
     Blitz/Parley reserve no space for an inline element's horizontal
     margin, border or padding: `AAA<span style="padding:0 20px">BBB</span>CCC`
     sets `AAABBBCCC` as one word here, while Chromium puts BBB at 75.3pt
     and CCC at 120.3pt (`border:2px; padding:3px 10px; margin:0 5px`: 73.0 /
     115.8). Painting a border without that space would draw over the text.
     It is open in Blitz's tracking issue DioxusLabs/blitz#256 ("Margin/
     border/padding for inline spans (should be bidi-aware)"). Spacer
     `InlineBox`es at the span's ends would not do: an in-flow box is a
     U+FFFC in the text, which adds line-break opportunities where there
     were none (`AAA|BBB`) and can leave the padding alone at a line end.
     Needs start/end spacing on Parley style spans (a Parley feature), then
     Blitz passing the span's margin+border+padding, then fulgur drawing the
     border box per line fragment (`box-decoration-break: slice`).
     **Upstream has a PR for it:** DioxusLabs/blitz#1018 ("Support margin,
     border and padding on non-atomic inline elements", open since
     2026-10-02, no review yet). Read on 2026-11-24: spans stay style spans;
     their inline-axis margin+border+padding go to Parley as start/end edges
     (RTL-aware), take space and wrap; `BaseDocument::inline_span_fragments`
     gives one border box per line (`box-decoration-break: slice`, ascent +
     descent + vertical padding/border, as Chrome); blitz-paint draws
     background + border per fragment. 12 of 15 probe cases match Chrome
     exactly; WPT +131 / −49. **It depends on DioxusLabs/parley#83**
     (`TextStyle::inline_start/inline_end`, `Line::span_fragments()`,
     `Layout::span_edges_mut()`), pulled from a Parley git branch; we use
     Parley 0.13 from crates.io. **Decision needed:** take #1018 + parley#83
     into our forks now (rebase onto `public-layout-pass`, conflicts in
     `document.rs`, `inline.rs`, blitz-paint; fulgur then draws span
     fragments instead of d6bc3673's per-run backgrounds), or wait for both
     to land upstream.
   - **Decision (2026-11-24): wait for upstream.** parley#83 sits on a
     pre-0.12 base (dd7e0b04); carried onto Parley 0.13.0 its main commit
     `cfefbd1d` conflicts in 9 files (33 hunks), mostly from
     linebender/parley#902 (inline boxes as U+FFFC) and later line
     breaking / shaping changes. Porting it is days of work that the next
     Parley change can undo, and both PRs are the Blitz maintainer's own.
     Asked about a rebase on blitz#1018 (issuecomment-7127133066); test a
     rebased branch when one appears.
7. **Still open from earlier:** e-irsaliye `hr` width (a colspan cell with
   percentages over 100%, 318 vs 310px); `::first-letter`; page CSS inside
   inline SVG. All three fixed in the forks (2026-11-24), drop caps aside.
   - **`hr` width: the cause is auto-width tables with percentage cells
     (2026-11-24).** Fixed-width tables (`width=400`, cells 60%+60%,
     70%+50%, 50%+40%+30%, a colspan `<hr>` row) match Chromium exactly.
     With `width: auto` they do not (first-row cell widths in px, 12px
     Arial, "aaaa aaaa" / "bbbb bbbb"):
     30%+30%: here 57/56, Chromium 95/94; 60%+40%: 68/45 wrapped, Chromium
     85/57 on one line; 50%+auto: 42/41 wrapped, Chromium 57/56; 60%+60%:
     68/45 wrapped, Chromium 85/57. Chromium (Blink
     `TableLayoutAlgorithmAuto`) makes the table's max width the largest of
     the columns' summed max-content, each percentage column's
     `max-content / percent` (percentages taken in order and cut so they
     total at most 100%), and the non-percentage columns'
     `max-content / (1 − total percent)`. Here percentages give the table
     no width and even the max-content is not reached.
   - **Fixed in the fork (2026-11-24):** Blitz 20d28619
     (`public-layout-pass`): `percent_max_width` gives an auto-width table
     that width; the four tables and the colspan `<hr>` (142px) now match
     Chromium exactly. Upstream: commit 9bc6f1a9 on DioxusLabs/blitz#1700
     (the PR that brought the auto table layout), with blitz-tests cases.
     Test `tests/tables.rs::percentage_cells_widen_an_auto_width_table`.
     The real e-irsaliye page (318 vs 310px) is not in the repo; recheck it
     when it is at hand.
   - **`::first-letter`: done except drop caps (2026-11-24).** Stylo
     computed it, Blitz never used it. Blitz cf4ca56e (`public-layout-pass`):
     an element with a `::first-letter` style gets a node holding it
     (`ElementData::first_letter`, synced like `::before`/`::after`); inline
     layout gives the first letter unit (punctuation around it, combining
     marks) a style span with that node as brush. Probe: red / 48px blue /
     floated green letters 18 / 344 / 205 colored pixels, Chromium 17 / 352
     / 204; the first two look the same. Test
     `tests/pdf_content.rs::first_letter_gets_its_own_style`.
   - Upstream: issue DioxusLabs/blitz#1739, PR #1740 (branch
     `pr/first-letter` on `main`, five blitz-tests cases).
   - **Next: drop caps.** A floated first letter is laid out inline: it
     needs a float box of its own. Blitz already floats inline content
     (`floats` feature: a floated child is a `CustomOutOfFlow` inline box,
     placed by `place_inline_float`). Plan:
     1. `flush_first_letter` (construct.rs): when the `::first-letter`
        style floats, give the first-letter node a text child holding the
        first letter unit of the block's first text (`split_first_letter`).
     2. Put that node first among the block's layout children (where
        `::before` goes), so it is laid out and fulgur draws it.
     3. `inline/build.rs`: for a floated first letter, push the node as a
        `CustomOutOfFlow` inline box instead of a style span, and leave the
        letter out of the first text.
     Test against the probe's green "C" (Chromium: text wraps beside it).
   - **Page CSS inside inline SVG: fixed in the fork (2026-11-24).**
     `.icon rect { fill }` / `#b { fill }` in the page's `<style>` drew
     black, and an SVG's own `<style>` did not cascade with the page
     (Chromium applies it document-wide). Stylo computes SVG styles right
     (Blitz maps the presentation attributes, `stylo/svg_attrs.rs`); the
     loss was `svg_source()`, the markup fulgur hands to usvg. Blitz
     4257684d (`public-layout-pass`): it writes each SVG element's computed
     fill/stroke family and visibility into its `style` attribute,
     `currentcolor` resolved. Paint servers (`url(#g)`) and
     `context-fill/stroke` stay as the markup has them (the computed URL is
     absolute and usvg cannot follow it), so a gradient set only by page CSS
     still does not apply. Eight probe cases equal Chromium. Test
     `tests/pdf_content.rs::page_css_styles_inline_svg`.
     Upstream: issue DioxusLabs/blitz#1741, PR #1742 (branch
     `pr/svg-source-computed-paint` on `main`, three blitz-tests cases).
     Blitz is moving inline SVG to a native renderer that reads computed
     styles (`native_svg`, tracking #1543), which makes this moot later.
8. **PDF/A: done** (`--pdf-a 1b|2b|2u|2a|3b|3u|3a`, fulgur `Config::pdf_a`,
   `tests/pdf_a.rs`). Every website sample passes veraPDF 1.30.4 at 2b and
   2a (2026-11-22). The roadmap lists what follows: PDF/A-3 embedded files,
   PDF/A-4, PDF/X-4, PDF/VT.
14. **PDF/UA-1: done (2026-11-22)**, `--pdf-ua` (`tests/pdf_ua.rs`, server
    option `pdf-ua`). fulgur 63dd6908 tags all page content: text in cells,
    divs and anonymous blocks, figures for img/svg (aria-label, title,
    role="img"), inline boxes in reading order, links (also image-only,
    inline boxes, margin boxes, opacity groups) each annotation in one Link,
    artifacts for decoration and @page boxes, ColSpan/RowSpan, aria-hidden,
    role="presentation" tables, heading level check, outline fallback. Tests
    in fulgur `pdf_ua_{content,structure,semantics,drawing}.rs`.
    - veraPDF PDF/UA-1 passes all 10 website samples and 26 edge-case pages
      (kept in the session scratchpad only). The newsletter sample's first
      `h3` became `h2 class="lede"` (same look) and the Chinese chart got
      `role="img"` + `aria-label`.
    - The work came from a five-lens review workflow (42 findings, 33
      verified three times each) and three parallel fixes.
    - Still open: content under fractional `opacity` is tagged as one
      sequence of that element (krilla cannot tag inside a transparency
      group), so headings/lists/tables inside it are flattened; PDF/UA +
      PDF/A-2a together (krilla applies one validator); PDF/UA-2; an
      `<img>` inside a link: checked 2026-11-24, an inline one is right
      (`P > Link > [Figure, OBJR]`), but **a block-level one gets no link at
      all**, tagged or not (`<a href><img style="display:block"></a>`: 0
      link annotations; the same image inline: 1). fulgur makes link areas
      only from inline runs (`paragraph.rs` `push_rect`), so any block box
      inside `<a>` (an image, a `display:block` card) is not clickable.
      Fixed for block-level images and SVGs (2026-11-24, fulgur 509a1489,
      PR #969 item 52: their box is a link area, their Figure gets a
      `Link` with the OBJR). A block-level `<a href>` (a
      `display:block` card) is clickable over its whole box since
      fulgur PR #969 item 53; `<caption>` as an anonymous P
      (fixed 2026-11-24: fulgur c8870762 tags it `Caption`, before its
      `Table` in the caption wrapper's Div; PR #969 item 49); `aria-labelledby` (fixed
      2026-11-24: fulgur 314812c6, figures and links take the named
      elements' text before `aria-label`; PR #969 item 50); an inline `<span role="img">` gave an
      empty Figure (fixed 2026-11-24: fulgur f0ad75c3 puts its text in the
      Figure, in place; PR #969 item 51).
    - Upstream: reported as fulgur-rs/fulgur#975; the fix is item 43 of
      PR #969 (item 42 is `snap_page_area_origin`).
15. **Running elements drawn twice: fixed in the fork (2026-11-23).** With
    `position: running()` in the page's own `<style>`, the element was drawn
    in the margin box AND in the body; fulgur hid it only for AssetBundle and
    `<link>` CSS (`cleaned_css`). Same on upstream main `bd6222fe`. fulgur
    65bb78cc (`blitz-master`): `RunningElementPass` marks each running
    element with `data-fulgur-running` and injects
    `[data-fulgur-running]{display:none !important}`; test
    `running_inline_style.rs`, three gcpm snapshot goldens regenerated.
    Upstream: fulgur-rs/fulgur#983, PR #969 item 46.
    Pinned here, with the test
    `tests/pdf_content.rs::running_elements_in_an_inline_style_are_drawn_once`.
    Only following the PR now.
    Repro:
    `<style>@page{@top-right{content:element(h)}} .h{position:running(h)}</style><div class=h>RUNHEAD</div><p>Body text</p>`.
16. **Ligature text: fixed (2026-11-22).** fulgur 19aa3185 maps each
    ligature glyph to all of its letters; before, `office` extracted, copied
    and read aloud as `ofce` (`tests/pdf_content.rs`, fulgur
    `ligature_text.rs`). Upstream: fulgur-rs/fulgur#976, PR #969 item 44.
    The benchmark's `ligature-text` check now uses Comfortaa through
    `@font-face`: with `font-family: serif` it fell on a font without
    ligatures on this machine and passed without testing anything.
    Found while testing the fix (each confirmed by two skeptics, all there
    before it; where each lives, fulgur, Blitz, Parley or krilla, is not
    checked yet):
    - Fixed in item 17: the ligature across a link edge, RTL reading order,
      element `lang`, the repeated heading `/T` and tabs in `<pre>`.
    - PDF/UA and PDF/A-u/a: krilla maps every glyph of a multi-glyph
      cluster (ligature + mark, Arabic harakat) to the whole cluster text.
      ActualText keeps pdftotext right; CMap-only readers see it twice.
    - Soft hyphen: no visible hyphen at the break, drawn with the space
      glyph, U+00AD and U+200B left in the text; a ligature across a soft
      hyphen removes that break.
    - `list-style-type: upper-roman`, `lower-greek` and `@counter-style`
      names draw `□`; `::marker { content }` is ignored; RTL list markers
      are drawn on the left.
    - `<input value>` and `<textarea>` text is not drawn; in Persian, the
      letter before a ZWNJ has no ToUnicode entry.
    - No effect: `::first-line` (`::first-letter` works since cf4ca56e,
      item 7, drop caps aside),
      `hyphens: auto`, `<wbr>`, `text-overflow: ellipsis`, `bdo` /
      `unicode-bidi: bidi-override`.
17. **PDF/UA text: fixed (2026-11-23).** fulgur 31f572d2:
    - Right-to-left lines are read in order: the content stream keeps the
      visual order (as Chromium), each run of such a line gets its own
      marked content, and the structure tree takes them in reading order.
    - An element's own `lang` is written as `/Lang` on its structure element,
      and on a marked-content Span for an inline element. Values are checked
      as language tags (`en_US` becomes `en-US`, others are dropped); an
      invalid `<html lang>` with `--pdf-ua` says so. A caption keeps the
      language of its table.
    - A preserved tab draws nothing and reaches the next tab stop (same
      positions as Chromium for `tab-size` numbers and lengths); PDF/UA and
      PDF/A accept it. Where stops would carry a line past its line box
      (shrink-to-fit cells, a float beside it, a `pre-wrap` line at the
      edge), or on centred, right-aligned and RTL lines, tabs keep their
      glyph width.
    - A glyph that joins a link's letter in a ligature goes to that link.
    - Heading `/T` holds the text once, without U+FFFC.
    - Tests: fulgur `pdf_ua_text.rs` (25), bdhtml2pdf `tests/pdf_ua.rs`.
      Two review rounds compared every case with the build
      before, pixel by pixel, and with Chromium.
    - Upstream: fulgur-rs/fulgur#977 (reading order, lang, `/T`), #978
      (tabs), #979 (links in ligatures), PR #969 item 45. krilla writes the
      `/ReversedChars` pieces of one RTL run in visual order:
      LaurenzV/krilla#492 (report only).
    - Left as they are: RTL lines inside a fractional-opacity group are one
      sequence in visual order (krilla cannot tag inside a group); a
      `::after` image on an RTL line is read after the first run; a
      ligature joining two links stays with the first; the tab stop
      interval uses the tab's own font, not the block's; `tab-size` on an
      inline element is ignored.
    - bdhtml2pdf: a stylesheet with non-ASCII text before `url()` or
      `@import` made `css_urls` panic (byte index inside a character);
      fixed with a unit test.
18. **Dependency forks (2026-11-23):** the core dependencies are forked
    under github.com/bahadirdogru (fulgur, blitz, taffy, stylo, html5ever,
    rust-cssparser, parley, harfrust, fontations, icu4x, wuff, woff2-rs,
    fontdb, krilla, pdf-writer, subsetter, hayro, resvg, tiny-skia, vello,
    anyrender, boa) in case one goes away; general-purpose crates of large
    teams are left out. `scripts/sync-forks.sh` fast-forwards their default
    branches to upstream (never forces, never touches our work branches);
    run it now and then, and fork any new core dependency.
19. **Boa speed (long term, 2026-11-23): on hold, follow upstream
    (2026-11-24).** Decision: keep Boa as it is; no VM work in our fork.
    Track Boa releases and boa-dev/boa#5686, re-measure the QR page on
    each Boa update. A user's e-document with a qrcode.js QR code takes 1.7–1.9 s
    per document vs 0.8 s in wkhtmltopdf, and they cannot change the HTML.
    Measured here (`tests/qrcode.rs` page, 61×61 code): 0.73 s with the code,
    0.10 s without; the `innerHTML` of the 436 KB table costs ~0.04 s, its
    layout ~0.1 s, the `offsetWidth` read nothing. The QR math itself takes
    ~0.55 s in Boa 0.22.0 vs 0.09 s in `node --jitless` and 0.03 s in node.
    Microbenchmarks (1.83 M iterations): a method call costs ~0.5 µs; the
    same `s += k & 1` loop takes 0.13 s inside a function and 1.6 s at
    global scope (12×).
    - **The global case is Boa itself (2026-11-24),** not the window setup.
      A bare `boa_engine` program (no Blitz): `var s` / `var k` at global
      scope 1.61 s, `let` 0.17 s, inside a function 0.16 s (0.22.0); Boa
      `main` 6801ee24: 1.40 / 0.19 / 0.12 s. callgrind: each global `var`
      read or write goes through the global object's ordinary
      [[Get]]/[[Set]]/[[DefineOwnProperty]] with a hashed `UniqueShape`
      lookup and string compares, ~6,700 instructions per iteration. No
      Boa issue covers it. A cache of global `var` bindings would fix it.
    - **It does not explain the QR code:** qrcode.min.js runs inside one
      `!function(){…}()`, so its helpers are closure variables, not
      globals. Its cost is property access and calls (~0.5 µs a call).
      Reported: boa-dev/boa#5686.
    - **QR code profile (2026-11-24):** qrcode.min.js with a stub
      `document` in a bare Boa program: Boa `main` 0.54 s, `node --jitless`
      0.094 s (5.7×), node 0.035 s, so Blitz adds nothing here. callgrind
      (5.16 G instructions) has no single hot spot: value drops and
      reference counting ~15% (`NanBoxedValue` drop glue 8.7%,
      `GcHeader::inc_ref_count` 1.6%, the `GC_DROPPING` thread-local read
      on every drop 2.3%), the dispatch loop `Context::run` ~9% plus
      register moves, property reads by name with the inline cache ~5%,
      `get_by_value` 1.4%. A 5× gain needs interpreter work across the
      board, not one fix. Checked the obvious candidate: `GC_DROPPING` is
      read only when a `Gc` pointer drops (not for numbers), from a
      `const`-initialised thread-local, ~2% of the run; not worth a PR on
      its own. Real gains need dispatch and value-handling work in Boa's
      VM (long term). Setup kept in the session
      scratchpad: `boabench` (a `boa_engine` program that evals a file) and
      `stub.js` + `run.js` around the vendored qrcode.min.js.
20. **Goal: PDF/VT-3 (2026-11-24).** Support ISO 16612-3 (PDF/VT-3, on
    PDF 2.0 and PDF/X-6): many personalized records (invoices, statements)
    in one file, each record a document part (DPart hierarchy with its
    metadata), and content shared between records (logos, backgrounds,
    fonts) stored once as XObjects and reused. In the README's goals.
    Steps:
    - PDF 2.0 output: check what krilla writes with `PdfVersion::Pdf20`.
    - PDF/X-6 first (output intent with an ICC profile, `TrimBox` /
      `BleedBox`, no RGB without an intent); krilla has no PDF/X
      validator, so the checks are ours (see docs/roadmap.md, PDF/X).
    - Records in: a command-line / server form for many records into one
      PDF (one HTML per record, or one template plus data), with the
      DPart tree and its record metadata.
    - Reuse: identical images and repeated page parts written once.
    - Validation: find a PDF/VT-3 checker (veraPDF has no PDF/VT profile;
      a commercial preflight or the ISO's conformance files).
9. **0.1.2 (2026-11-23):** released without a benchmark file of its own:
   this machine was out of memory, and the user runs the five-engine
   benchmark on another machine. That run came on 2026-11-23
   (docs/benchmarks/2026-11-23-i7-12800h.md, all five engines on the
   i7-12800H); README, the website, docs/why.md and docs/security.md now
   quote only it. The archives are now named
   `linux-x86_64`, `linux-aarch64`, `macos-aarch64`, `windows-x86_64`, and
   the install commands in README.md and docs/index.html use them.
   **0.1.3 (2026-11-23):** the same code built with `opt-level = 3` instead
   of `"s"` (29 MB instead of 22 MB, JavaScript pages 12–14% faster), no
   benchmark file of its own. The prose now says 29 MB; the comparison
   tables in README.md and docs/index.html (install size 22 MB, the
   `bar s22` chart) still quote the 0.1.2 run. Replace them with the next
   five-engine run, as one run of all engines.
10. **Decided, not planned:** Canvas 2D (an environment with a canvas can
    rasterize it) and the wkhtmltopdf header/footer flags (the page sets
    them with `@page` margin boxes).
11. **crates.io: blocked until upstream is published.** The name
    `bdhtml2pdf` is free and Cargo.toml has the metadata (repository,
    homepage, keywords, categories). `cargo publish` refuses path and git
    dependencies, and we use both: fulgur from `../fulgur` (crates.io has
    0.41.0 without our port), Blitz from git plus `[patch]`, and
    `blitz-vibey-script` is not on crates.io at all. Possible once fulgur
    publishes the Blitz-master port and Blitz publishes vibey-script; until
    then the GitHub release binaries are the install path. GitHub topics and
    homepage are set (2026-11-22).
12. **Comparison on one machine (2026-11-22):** all five engines (bdhtml2pdf,
    sghtmltopdf 0.8.0, wkhtmltopdf 0.12.6.1, Chromium 156, WeasyPrint 70.0)
    on the i7-6700 with Pardus 25, the same system packages and fonts,
    `bench/run.sh -n 10` plus `bench/security.sh`: docs/benchmarks/2026-11-22.md.
    README, the website and docs/why.md, docs/security.md now quote only this
    run. The user asked that every comparison use the same packages: rerun
    all engines together on one machine, never mix runs. The engines were
    unpacked into the session scratchpad (sghtmltopdf with `cargo install
    --root`, wkhtmltopdf from the official bookworm .deb with `dpkg-deb -x`);
    GNU time is not installed here, so `TIME=` points to an extracted `time`
    package. WeasyPrint is not in `bench/security.sh` yet.
13. **Server and bindings (2026-11-22):** `bdhtml2pdf serve` (src/server.rs,
    tests/server.rs) and `bindings/` for Python, Node.js, Ruby and PHP, tested
    in the CI job `bindings`. Publishing to PyPI, npm, RubyGems and Packagist
    is **deferred by the user (2026-11-22)**; do not start it or ask for
    tokens until they bring it up. Also waits with it: the binary inside the
    wheels and npm packages. All four test suites also pass locally (Ruby
    3.3, PHP 8.4), and the documented installs work: `pip install git+…`,
    `npm install ./…`, the Composer path repository and the Gemfile `git:` +
    `glob:` line (Bundler 2.6).

## How to validate PDF/A and PDF/UA

veraPDF 1.30.4 is installed in `~/.local/opt/verapdf` (no wrapper script):

```sh
java -jar ~/.local/opt/verapdf/bin/cli-1.30.4.jar --flavour ua1 --format text out.pdf
java -jar ~/.local/opt/verapdf/bin/cli-1.30.4.jar --flavour 2b --format text out.pdf
```

`--format xml` lists each failed rule with its clause and context.

## How to compare with Chromium

```sh
target/release/bdhtml2pdf -q page.html bd.pdf
chromium-browser --headless=new --disable-gpu --no-pdf-header-footer \
  --print-to-pdf=cr.pdf file://$PWD/page.html
pdftoppm -f 1 -l 1 -r 96 -png -singlefile bd.pdf bd   # 1 px = 1 CSS px
pdftoppm -f 1 -l 1 -r 96 -png -singlefile cr.pdf cr
```

Measure edges with Pillow on solid-colored probe boxes rather than by eye.

## Rules

- Commits and PRs carry no co-author trailers. Author: Bahadır
  <dogrubahadir@gmail.com>. Check `git log @{u}.. --format=%B` before pushing.
- Run fulgur workspace tests with `-j 4` and `--no-fail-fast`. With
  `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0` the fulgur target
  is about 7 GB instead of more than 14 GB; the disk here is nearly full.
- fulgur `cargo test --workspace`: only `fulgur-vrt` fails. Run it with
  `FONTCONFIG_FILE=$PWD/examples/.fontconfig/fonts.conf` (44 of 65 differ
  then, 45 without it). The goldens are upstream fulgur's, on the old Blitz.
  Against the Chromium goldens in `goldens/chrome` (34 of the 44 have one),
  our output is closer to Chromium in 20, the same in 14 and further in
  none; the 10 without one are GCPM features Chromium lacks. So the
  goldens are out of date, not our output. The PR asks the maintainers
  whether to regenerate them.
- Fonts without sudo: `apt-get download fonts-noto-cjk fonts-noto-cjk-extra
  fonts-vazirmatn`, `dpkg-deb -x` each, copy the `.ttc`/`.ttf`/`.otf` files
  to `~/.local/share/fonts/debian-pkgs`, `fc-cache -f`. Done on this machine.
