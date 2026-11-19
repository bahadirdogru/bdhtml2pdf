//! bdhtml2pdf: HTML + JavaScript → vector PDF.
//!
//! Blitz master + Boa runs the page's JavaScript and produces the final DOM;
//! fulgur (ported to Blitz master, same Stylo) lays it out into a vector PDF.

pub mod depth;
pub mod fetch;
pub mod lifecycle;
pub mod limits;
pub mod page;
pub mod pdf;
pub mod prefs;

pub use lifecycle::{JsOptions, JsResult, JsTimeout, run_scripts, run_scripts_bounded};
pub use page::PageOptions;
pub use pdf::render_pdf;
