//! Long tables in a page with a `lang` attribute must not be slower than the
//! same table without it.
//!
//! fontique (Parley's font library) stores the system's fallback fonts only
//! for (script, language) pairs it has language-specific data for. Latin text
//! in `lang="tr"` is not one of them, and Parley asks for Han fallbacks after
//! every item's own script, so each table cell used to query fontconfig again:
//! 1000 rows took 1.1 s with `lang="tr"` and 0.12 s without. Blitz now looks
//! the default fallbacks up when a document is created. The test compares the
//! two pages instead of using a fixed time limit, so it does not depend on the
//! machine's speed.

use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

fn table(lang: &str, rows: usize) -> String {
    let mut html = format!(
        "<!DOCTYPE html><html{lang}><head><meta charset=utf-8><style>\
         table{{width:100%;border-collapse:collapse}}td{{padding:2px 4px}}\
         </style></head><body><table>"
    );
    for i in 1..=rows {
        html.push_str(&format!(
            "<tr><td>Satır {i} — eğitim modülü</td><td>{}</td><td>1,25 ₺</td></tr>",
            i % 7 + 1
        ));
    }
    html.push_str("</table></body></html>");
    html
}

/// The fastest of three conversions.
fn convert_time(input: &Path, output: &Path) -> Duration {
    (0..3)
        .map(|_| {
            let start = Instant::now();
            let o = Command::new(env!("CARGO_BIN_EXE_bdhtml2pdf"))
                .args(["-q", "-n"])
                .arg(input)
                .arg(output)
                .output()
                .unwrap();
            assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
            start.elapsed()
        })
        .min()
        .unwrap()
}

#[test]
fn lang_attribute_does_not_slow_down_long_tables() {
    let dir = std::env::temp_dir().join(format!("bdhtml2pdf-tables-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let output = dir.join("out.pdf");
    let mut times = Vec::new();
    for (name, lang) in [("none", ""), ("tr", " lang=\"tr\"")] {
        let input = dir.join(format!("{name}.html"));
        std::fs::write(&input, table(lang, 500)).unwrap();
        times.push(convert_time(&input, &output));
    }
    let (plain, turkish) = (times[0], times[1]);
    assert!(
        turkish < plain * 2,
        "lang=\"tr\" took {turkish:?}, without lang {plain:?}"
    );
}
