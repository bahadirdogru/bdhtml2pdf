//! Column widths of tables with automatic layout, as Chromium sizes them
//! (CSS Tables 3's width distribution), with percentage widths on cells
//! honoured.
//!
//! Each test renders a table with coloured cells to PNG and measures where the
//! colours change.

use std::path::PathBuf;
use std::process::Command;

use image::{GenericImageView, Rgba};

fn tempdir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("bdhtml2pdf-tables-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Renders `table` 400 px wide and returns the widths of the runs of red,
/// green and blue along the table's first pixel row.
fn column_widths(name: &str, table: &str) -> Vec<(char, u32)> {
    let img = render(name, table);
    let mut runs: Vec<(char, u32)> = Vec::new();
    for x in 0..img.width() {
        push(&mut runs, colour(img.get_pixel(x, 0)));
    }
    runs.retain(|(c, _)| *c != '-');
    runs
}

/// The runs of colours down pixel column `x`, as `column_widths` gives them
/// across, black (`k`) included.
fn column_runs(name: &str, table: &str, x: u32) -> Vec<(char, u32)> {
    let img = render(name, table);
    let mut runs: Vec<(char, u32)> = Vec::new();
    for y in 0..img.height() {
        push(&mut runs, colour(img.get_pixel(x, y)));
    }
    runs
}

fn push(runs: &mut Vec<(char, u32)>, c: char) {
    match runs.last_mut() {
        Some((last, n)) if *last == c => *n += 1,
        _ => runs.push((c, 1)),
    }
}

fn colour(p: Rgba<u8>) -> char {
    match p.0[..3] {
        [r, g, b] if r > 200 && g < 60 && b < 60 => 'r',
        [r, g, b] if g > 200 && r < 60 && b < 60 => 'g',
        [r, g, b] if b > 200 && r < 60 && g < 60 => 'b',
        [r, g, b] if r < 60 && g < 60 && b < 60 => 'k',
        _ => '-',
    }
}

/// Renders `table` 400 px wide to PNG.
fn render(name: &str, table: &str) -> image::DynamicImage {
    let dir = tempdir(name);
    let input = dir.join("page.html");
    std::fs::write(
        &input,
        format!(
            "<!DOCTYPE html><meta charset=utf-8><style>body{{margin:0;font:10px Arial}}\
             table{{border-spacing:0}} td{{padding:0;vertical-align:top}}\
             .r{{background:rgb(255,0,0)}} .g{{background:rgb(0,255,0)}} .b{{background:rgb(0,0,255)}}\
             </style>{table}"
        ),
    )
    .unwrap();
    let out = dir.join("out.png");
    let o = Command::new(env!("CARGO_BIN_EXE_bdhtml2pdf"))
        .args(["-q", "--width", "400"])
        .arg(&input)
        .arg(&out)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    image::open(&out).unwrap()
}

fn words(n: usize) -> String {
    vec!["lorem"; n].join(" ")
}

#[test]
fn long_text_column_gets_the_space_short_columns_wrap() {
    // An invoice line: a short "2 pieces" column and a long description in a
    // 300 px table. Browsers give the description most of the width (it is
    // far from its max-content width), so "2 pieces" wraps; sharing space
    // equally (as CSS grid does) let the short column reach its max-content
    // width and squeezed the description.
    let runs = column_widths(
        "invoice",
        &format!(
            "<table width=300><tr><td class=r>2 pieces</td><td class=b>{}</td></tr></table>",
            words(60)
        ),
    );
    let [('r', short), ('b', long)] = runs[..] else {
        panic!("{runs:?}");
    };
    assert_eq!(short + long, 300, "{runs:?}");
    assert!(short < 35, "the short column wraps: {runs:?}");
}

#[test]
fn columns_grow_from_min_to_max_content_by_the_same_fraction() {
    // As in Chromium (CSS Tables 3): in a table narrower than its content,
    // each auto column gets the same fraction of the way from its min-content
    // to its max-content width. Here min/max are 50/100 and 40/400, and the
    // 210 px above the min-content widths go 50 : 360, making 75.6 and 224.4.
    // WebKit (wkhtmltopdf) shares the width in proportion to max-content
    // widths instead (60 and 240), so long text takes more of the table.
    let block =
        |w: u32| format!("<div style='display:inline-block;width:{w}px;height:10px'></div> ");
    let runs = column_widths(
        "fraction",
        &format!(
            "<table width=300><tr><td class=r style='font-size:0'>{}</td>\
             <td class=b style='font-size:0'>{}</td></tr></table>",
            block(50).repeat(2),
            block(40).repeat(10)
        ),
    );
    let [('r', a), ('b', b)] = runs[..] else {
        panic!("{runs:?}");
    };
    assert_eq!(a + b, 300, "{runs:?}");
    assert!((75..=76).contains(&a), "{runs:?}");
}

#[test]
fn percentage_widths_on_cells_are_honoured() {
    let runs = column_widths(
        "percent",
        &format!(
            "<table width=400><tr><td class=r width='25%'>a</td>\
             <td class=g>{}</td><td class=b width='10%'>b</td></tr></table>",
            words(80)
        ),
    );
    let [('r', a), ('g', text), ('b', b)] = runs[..] else {
        panic!("{runs:?}");
    };
    assert_eq!((a, text, b), (100, 260, 40), "{runs:?}");
}

#[test]
fn spare_width_goes_to_columns_in_proportion_to_their_content() {
    // A 400 px table whose content is narrower: each column grows in
    // proportion to its max-content width (here 1 : 3).
    let runs = column_widths(
        "spare",
        "<table width=400><tr><td class=r><div style='width:30px;height:10px'></div></td>\
         <td class=b><div style='width:90px;height:10px'></div></td></tr></table>",
    );
    let [('r', a), ('b', b)] = runs[..] else {
        panic!("{runs:?}");
    };
    assert_eq!((a, b), (100, 300), "{runs:?}");
}

#[test]
fn auto_width_table_is_as_wide_as_its_content() {
    let runs = column_widths(
        "auto",
        "<table><tr><td class=r><div style='width:30px;height:10px'></div></td>\
         <td class=b><div style='width:90px;height:10px'></div></td></tr></table>",
    );
    assert_eq!(runs, [('r', 30), ('b', 90)]);
}

#[test]
fn table_narrower_than_its_content_is_widened() {
    // `width=100` with a 150 px wide cell: browsers make the table as wide as
    // its columns' min-content widths instead of letting the cell overflow.
    let runs = column_widths(
        "narrow",
        "<table width=100><tr><td class=r><div style='width:150px;height:10px'></div></td>\
         <td class=b><div style='width:20px;height:10px'></div></td></tr></table>",
    );
    assert_eq!(runs, [('r', 150), ('b', 20)]);
}

#[test]
fn fixed_width_cells_of_a_nested_table_shrink_to_fit_their_column() {
    // Turkish e-waybill templates put a table of `width:469px` cells in a 40%
    // column of an 800 px table. A cell's width does not raise its column's
    // min-content width in browsers, so the nested table shrinks to the 40%
    // and the other columns keep their percentages.
    let runs = column_widths(
        "nested-fixed",
        "<table width=400><tr><td class=r width='40%'>\
         <table><tr><td style='width:300px'>a</td></tr></table></td>\
         <td class=b width='60%'>b</td></tr></table>",
    );
    assert_eq!(runs, [('r', 160), ('b', 240)]);
}

#[test]
fn lines_in_a_middle_aligned_cell_are_as_tall_as_in_a_block() {
    // A cell's own `vertical-align` (`middle` from the UA style sheet's rows)
    // places the cell's content in the row; applied to the cell's text as
    // well, it made each line ending in `<br>` taller than in a block.
    let lines = "x<br>x<br>x<br>x";
    let in_cell = column_runs(
        "br-cell",
        &format!(
            "<table><tr><td class=r style='vertical-align:middle;width:100px'>{lines}</td></tr></table>"
        ),
        50,
    );
    let in_block = column_runs(
        "br-block",
        &format!("<div class=r style='width:100px'>{lines}</div>"),
        50,
    );
    assert_eq!(
        in_cell[0], in_block[0],
        "cell {in_cell:?}, block {in_block:?}"
    );
}

#[test]
fn collapsed_borders_take_one_border_between_rows() {
    // With `border-collapse: collapse` two rows share one 1 px border: three
    // 10 px rows make a 34 px table. Counting each cell's border inside the
    // cell as well made every row 2 px taller.
    let runs = column_runs(
        "collapse",
        "<table style='border-collapse:collapse;width:100px'>\
         <tr><td class=r style='border:1px solid #000'><div style='height:10px'></div></td></tr>\
         <tr><td class=r style='border:1px solid #000'><div style='height:10px'></div></td></tr>\
         <tr><td class=r style='border:1px solid #000'><div style='height:10px'></div></td></tr>\
         </table>",
        50,
    );
    assert_eq!(
        runs[..7],
        [
            ('k', 1),
            ('r', 10),
            ('k', 1),
            ('r', 10),
            ('k', 1),
            ('r', 10),
            ('k', 1)
        ],
        "{runs:?}"
    );
}
