# bdhtml2pdf – Aşama 1 Araştırma Notları

Tarih: 2026-11-18
Yöntem: Aday depolar shallow clone ile indirildi; README, Cargo.toml, lisans ve
git geçmişi okundu. **Derleme ve RAM/binary ölçümü yapılmadı** (makinede Rust
toolchain yok). Aşağıdaki boyut/RAM rakamları projelerin kendi iddialarıdır.

## Hedef

wkhtmltopdf'in modern halefi: tek binary, tek process, düşük RAM, vektör PDF,
JavaScript destekli, Chromium/WebKit/Qt yok. Yazılmış hiçbir şey yeniden
yazılmayacak; iş mevcut parçaları birleştirmek.

## Bulgular

| Proje | Ne yapıyor | JS | PDF | Lisans | Son commit |
|---|---|---|---|---|---|
| [fulgur](https://github.com/fulgur-rs/fulgur) | "Modern wkhtmltopdf alternatifi". Blitz 0.2 + Stylo 0.8 + Taffy 0.9 + Parley 0.6 → Krilla 0.7 | **Yok** (tasarım dokümanında açıkça non-goal) | Vektör, sayfalama, GCPM header/footer, sayfa no, font subsetting, PDF/UA, bookmark | MIT/Apache-2.0 | 2026-11-08, çok aktif |
| [Blitz](https://github.com/DioxusLabs/blitz) | html5ever + Stylo + Taffy + Parley üzerinde DOM/stil/layout | **Var:** `blitz-vibey-script` paketi (Boa ile blitz-dom'a doğrudan DOM, event, timer, fetch binding; Preact çalıştırabildiği belirtiliyor). `publish = false`, ~9.3k satır | Yok (vello ile raster paint) | MIT/Apache-2.0 | 2026-11-18, çok aktif, 0.3.0-beta |
| [OxiBrowser](https://github.com/project-oxi/oxibrowser) | AI ajanları için headless tarayıcı. Boa + Blitz 0.3 + Stylo + Parley, CDP | Var (Boa, Shadow DOM, fetch/XHR, Canvas 2D) | **Raster**: vello_cpu ile PNG → printpdf ile PDF'e gömme | MIT | 2026-09-30 |
| [TypePress](https://github.com/alitrack/typepress) | fulgur üzerine Markdown/Mermaid/KaTeX ön-işleme | Yok | fulgur | MIT | 2026-06-24 |
| [Krilla](https://github.com/LaurenzV/krilla) | Yüksek seviye PDF yazıcı | – | Vektör, font subsetting | MIT/Apache-2.0 | 2026-10-04 |
| [litehtml](https://github.com/litehtml/litehtml) | C++ hafif HTML/CSS layout | Yok | Yok (çizim host'a ait) | BSD-3 | 2026-10-25 |
| [waka/sghtmltopdf](https://github.com/waka/sghtmltopdf) | Spesifikasyondaki "waka/sg". Saf Rust, kendi CSS motoru (html5ever + cssparser + selectors + Taffy + pdf-writer); wkhtmltopdf/wicked_pdf yerine | **Yok** | Vektör, streaming modu, sunucu modu, Ruby gem | MIT | v0.8.0 (ilk aramada adı eksik yazıldığı için bulunamamıştı) |

Ek notlar:
- fulgur, Blitz'in yerine kendi layout motoru **Raikiri**'yi geliştiriyor
  (`fulgur-raikiri`, "under development"). Blitz backend'i şu an varsayılan.
- fulgur, Blitz **0.2** / Stylo **0.8** kullanıyor; Blitz'in JS paketi ise
  Blitz **0.3** üzerinde. İkisini birleştirmek için fulgur'un Blitz adaptörünün
  0.3'e taşınması gerekir.
- OxiBrowser'ın PDF'i raster olduğu için (seçilemeyen metin, büyük dosya)
  PDF tarafı için uygun değil; JS-DOM entegrasyonu için referans olabilir.
- litehtml JS ve PDF içermediği için C++ yolu her şeyi (JS bağlama, PDF) ayrıca
  yazmayı gerektirir. "Sıfırdan yazma" kuralıyla çelişir; elendi.

## Sonuç: Önerilen mimari

Neredeyse tüm parçalar zaten var. Eksik olan yalnızca **yapıştırıcı**:

```
HTML ──► blitz-html/blitz-dom (Stylo + Taffy + Parley)
            │
            ├─► blitz-vibey-script (Boa): <script> çalıştır, DOM'u değiştir,
            │    timer/microtask kuyruğunu boşalt, "hazır" sinyali bekle
            │
            └─► fulgur-blitz: sayfalama + GCPM → Krilla → vektör PDF
                         │
                 bdhtml2pdf CLI (wkhtmltopdf uyumlu argümanlar)
```

Yazılacak yeni kod:
1. fulgur'un Blitz adaptörünü Blitz 0.3'e taşımak (veya fulgur'a upstream PR).
2. Render yaşam döngüsü: script çalıştır → event loop'u sınırla (timeout,
   heap limiti) → `window.status`/`--window-status` benzeri hazır sinyali →
   layout → PDF.
3. wkhtmltopdf CLI uyumluluk katmanı (`--page-size`, `--orientation`,
   `--margin-*`, `--header-*`, `--footer-*`, `--javascript-delay`,
   `--window-status`, `--no-images` vb.).

Strateji: Mümkünse **fork değil, bağımlılık + upstream katkı**. Değişiklikler
fulgur ve Blitz'e PR olarak gönderilirse bakım yükü bizde kalmaz.

## Riskler / açık sorular

- `blitz-vibey-script` yayınlanmamış (`publish = false`) ve adı deneysel
  olduğunu düşündürüyor. Git bağımlılığı olarak kullanılabilir ama API'si
  değişebilir.
- Boa'nın React/Vue/Chart.js/ECharts performansı ve RAM'i ölçülmeli. Yetersizse
  QuickJS-NG (rquickjs) alternatif; ama Blitz binding'leri o zaman yeniden
  yazılmalı.
- Canvas 2D: Blitz/vibey-script'te `getContext` bulunamadı. Chart.js/ECharts
  için gerekli; OxiBrowser'ın Canvas 2D implementasyonu ve ECharts'ın SVG
  renderer modu incelenmeli.
- fulgur'un Raikiri'ye geçişi Blitz backend'ini ileride ikinci plana atabilir.
- Taffy'de flex/grid desteği var; TypePress README'sindeki "flex/grid yok"
  ifadesi eski görünüyor, test edilerek doğrulanmalı.

## Engel: fulgur ile Blitz master aynı binary'de olamaz (2026-11-18)

Stylo, `links = "servo_style_crate"` tanımlıyor; bu yüzden fulgur'un Stylo 0.8'i ile
Blitz master'ın Stylo 0.24'ü aynı binary'ye girmiyor. "Serileştirme köprüsü" (A planı)
tek process'te mümkün değil. fulgur'un `prep/blitz-0.3` dalı (2026-07, Blitz
0.3.0-alpha.6) ana dala alınmamış. fulgur main'i Blitz master'a (b4ad543) çevirince
`fulgur-blitz` **680 derleme hatası** veriyor: çoğu mekanik (`final_layout` alan→metot,
lifetime parametreleri, cssparser `new_error`, Stylo `clone_*`).

## Blitz vs OxiBrowser karşılaştırması (2026-11-18)

Betik: `bench/compare.sh`. Aynı fixture'lar (`tests/fixtures/`), release derleme,
peak RSS `/usr/bin/time`. Blitz = bu depodaki prototip (blitz-vibey-script, rev b4ad543).
OxiBrowser = main, `--features browser`, release.

| Fixture | Blitz süre / RSS | Blitz sonuç | OxiBrowser süre / RSS | OxiBrowser sonuç |
|---|---|---|---|---|
| modern-js | 0.08 s / 25 MB | 18/19 (`Intl` yok) | 0.25 s / 51 MB | 17/19 (`dataset`, `DocumentFragment.append` yok) |
| React 18 | 0.22 s / 42 MB | render + `useEffect`/timer güncellemesi OK, 50 satır | 0.28 s / 68 MB | **render yok**: `React is not defined` (UMD global kurulmuyor) |
| Vue 3 | 0.21 s / 45 MB | render + `mounted`/timer OK, 30 li | 0.30 s / 72 MB | **render yok**: `SVGElement is not defined` |
| Preact | 0.09 s / 26 MB | render OK | 0.17 s / 51 MB | **render yok**: `preact is not defined` |

| | Blitz prototip | OxiBrowser |
|---|---|---|
| Binary | **19.5 MB** | **68.8 MB** (README iddiası ~44 MB) |
| C/C++ bağımlılığı | yok | BoringSSL (`btls-sys`), `g++` gerekiyor |
| DOM | tek DOM (Blitz), JS ve layout aynı ağaç | ayrı JS DOM (~31k satır) → HTML'e serileştirilip Blitz'te yeniden parse |
| PDF | henüz yok | raster (PNG ekran görüntüsü sayfalara kesiliyor), yalnız CDP üzerinden |
| Diğer | — | `file://` yok; göreli `<script src>` çözülmüyor; `outerHTML`/`innerHTML` `undefined` |

Not: OxiBrowser'ın binary'si CDP, ağ, ajan gibi özellikleri de içeriyor; bunlar
sökülerek küçültülebilir. Ama React/Vue/Preact hataları JS DOM çekirdeğinde ve
sökme ile düzelmez.

## Sonraki adımlar

1. Rust toolchain kur; fulgur CLI'yi release modda derle, binary boyutu ve
   örnek fatura/tablo sayfalarında peak RSS ölç.
2. Blitz workspace'inde `blitz-vibey-script` testlerini çalıştır; Preact ve
   basit bir Chart.js (SVG/Canvas) sayfasıyla dene.
3. Küçük prototip: `ScriptDocument` → script çalıştır → serialize edilmiş
   DOM'u fulgur'a ver (adaptör taşınana kadar geçici yol) → PDF.

## sghtmltopdf karşılaştırması (2026-11-19)

sghtmltopdf 0.8.0 `cargo install` ile derlendi ve aynı fixture'larla ölçüldü
(`bench/run.sh`, `bench/security.sh`; sonuçlar `docs/benchmarks/`). Özet:

- Statik HTML için çok hafif ve hızlı (binary 12.6 MB, birkaç MB bellek, onlarca ms).
- JavaScript yok: React/Vue/Preact sayfaları boş ya da ham şablon olarak çıkıyor.
- CSS kendi yazdığı alt küme: `@layer`, `:nth-child(of S)`, gradient, çok sütun ve
  `lang="tr"` büyük/küçük harf dönüşümü yok (`istanbul` → `ISTANBUL`, `IŞIK` → `işik`).
- CJK metinde fontu alt kümelemeden gömüyor: `fonts.html` 19 MB PDF üretti.
- Güvenlik varsayılanları iyi: ağ yok, `--allow-path` dışı dosya yok.

Sonuç: sghtmltopdf statik şablonlar için iyi bir seçenek; bizim hedefimiz olan
modern JS + modern CSS için yeterli değil. Alınacak fikirler (kod değil): sunucu
modu ve sayfa sayfa streaming çıktı.
