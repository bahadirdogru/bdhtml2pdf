# Test fonts

Both fonts are under the SIL Open Font License 1.1 and are used only as test inputs.

| File | Font | Source | License |
|---|---|---|---|
| `Comfortaa-Regular.otf` | Comfortaa 3.x (OpenType/CFF) | Fedora package `aajohan-comfortaa-fonts`, upstream <https://github.com/alexeiva/comfortaa> | OFL 1.1, see `OFL-Comfortaa.txt` |
| `NotoSans-Regular.woff2` | Noto Sans (WOFF2) | copied from fulgur's test fixtures, upstream <https://github.com/notofonts/latin-greek-cyrillic> | OFL 1.1, <https://openfontlicense.org> |

The fixtures load them under made-up family names (`Brand`, `BrandOTF`,
`BrandWOFF2`), so a face can only end up in the PDF through the `@font-face`
rule and never through a system font of the same name.
