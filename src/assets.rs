//! Images the page refers to, registered with fulgur.
//!
//! fulgur draws only images that are registered in its [`AssetBundle`]; it
//! does not read `<img src>` from disk by itself. bdhtml2pdf finds the image
//! references a page makes (`<img src>`, `url()` in `style` attributes,
//! `<style>` elements and linked stylesheets) and registers the files that
//! lie inside the input directory. Everything outside it stays unreadable,
//! like every other resource.
//!
//! fulgur looks `<img src>` up by the attribute value as written, and CSS
//! `url()` by the path Stylo resolved, relative to the base directory. Each
//! file is registered under the key its reference will be looked up by.

use std::collections::HashSet;
use std::ops::Deref;
use std::path::{Path, PathBuf};

use blitz_dom::DocumentConfig;
use blitz_html::HtmlDocument;
use fulgur::AssetBundle;
use url::Url;

/// How deep `@import` chains are followed.
const MAX_IMPORT_DEPTH: usize = 16;

struct Collector {
    base_dir: PathBuf,
    base_url: Url,
    bundle: AssetBundle,
    registered: HashSet<String>,
    stylesheets: HashSet<PathBuf>,
}

impl Collector {
    /// The file `url` points to, if it is a regular file inside the base
    /// directory (after resolving `..` and symlinks).
    fn local_file(&self, url: &Url) -> Option<PathBuf> {
        if url.scheme() != "file" {
            return None;
        }
        let path = url.to_file_path().ok()?.canonicalize().ok()?;
        (path.starts_with(&self.base_dir) && path.is_file()).then_some(path)
    }

    fn register(&mut self, key: &str, url: &Url) {
        if key.is_empty() || self.registered.contains(key) {
            return;
        }
        if let Some(path) = self.local_file(url)
            && self.bundle.add_image_file(key, &path).is_ok()
        {
            self.registered.insert(key.to_string());
        }
    }

    /// `<img src>`: looked up by the attribute value.
    fn image_src(&mut self, src: &str) {
        let src = src.trim();
        if let Ok(url) = self.base_url.join(src) {
            self.register(src, &url);
        }
    }

    /// CSS `url()`: looked up by the resolved URL relative to the base.
    fn css_url(&mut self, reference: &str, css_base: &Url) {
        let Ok(url) = css_base.join(reference) else {
            return;
        };
        if let Some(key) = url.as_str().strip_prefix(self.base_url.as_str()) {
            let key = key.to_string();
            self.register(&key, &url);
        }
    }

    fn css(&mut self, text: &str, css_base: &Url, depth: usize) {
        for reference in css_urls(text) {
            if reference.is_import {
                self.stylesheet(&reference.url, css_base, depth + 1);
            } else {
                self.css_url(&reference.url, css_base);
            }
        }
    }

    fn stylesheet(&mut self, href: &str, from: &Url, depth: usize) {
        if depth > MAX_IMPORT_DEPTH {
            return;
        }
        let Ok(url) = from.join(href.trim()) else {
            return;
        };
        let Some(path) = self.local_file(&url) else {
            return;
        };
        if !self.stylesheets.insert(path.clone()) {
            return;
        }
        if let Ok(text) = std::fs::read_to_string(&path) {
            self.css(&text, &url, depth);
        }
    }
}

struct CssReference {
    url: String,
    is_import: bool,
}

/// The `url(…)` and `@import "…"` references in a stylesheet. A small
/// scanner, not a parser: it skips comments and handles quoted and unquoted
/// URLs, which is all that is needed to find file names.
fn css_urls(css: &str) -> Vec<CssReference> {
    let bytes = css.as_bytes();
    let lower = css.to_ascii_lowercase();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i..].starts_with(b"/*") {
            i = css[i + 2..]
                .find("*/")
                .map_or(bytes.len(), |e| i + 2 + e + 2);
            continue;
        }
        let rest = &lower[i..];
        let (is_import, skip) = if rest.starts_with("@import") {
            (true, "@import".len())
        } else if rest.starts_with("url(") {
            (false, 0)
        } else {
            i += 1;
            continue;
        };
        let mut j = i + skip;
        while j < bytes.len() && bytes[j].is_ascii_whitespace() {
            j += 1;
        }
        let in_url = lower[j..].starts_with("url(");
        if in_url {
            j += 4;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
        }
        let (value, end) = match bytes.get(j) {
            Some(&q @ (b'"' | b'\'')) => {
                let close = css[j + 1..]
                    .find(q as char)
                    .map_or(bytes.len(), |e| j + 1 + e);
                (&css[j + 1..close.min(bytes.len())], close + 1)
            }
            _ if in_url => {
                let close = css[j..].find(')').map_or(bytes.len(), |e| j + e);
                (css[j..close].trim(), close + 1)
            }
            _ => ("", j + 1),
        };
        if !value.is_empty() && !value.starts_with("data:") {
            out.push(CssReference {
                url: value.to_string(),
                is_import,
            });
        }
        i = end.max(i + 1);
    }
    out
}

/// Registers the images `html` refers to that lie inside `base_dir`.
pub fn collect_images(html: &str, base_dir: &Path) -> AssetBundle {
    let empty = AssetBundle::new();
    let lower = html.to_ascii_lowercase();
    if !["<img", "url(", "<link", "@import"]
        .iter()
        .any(|p| lower.contains(p))
    {
        return empty;
    }
    let Ok(base_dir) = base_dir.canonicalize() else {
        return empty;
    };
    let Ok(base_url) = Url::from_directory_path(&base_dir) else {
        return empty;
    };
    let mut c = Collector {
        base_dir,
        base_url,
        bundle: AssetBundle::new(),
        registered: HashSet::new(),
        stylesheets: HashSet::new(),
    };

    // The default configuration has no network provider, so nothing is
    // fetched here; the base URL makes relative `@import`s resolvable.
    let config = DocumentConfig {
        base_url: Some(c.base_url.to_string()),
        ..DocumentConfig::default()
    };
    let doc = HtmlDocument::from_html(html, config);
    let doc = doc.deref();
    let mut stack = vec![doc.root_node().id];
    while let Some(id) = stack.pop() {
        let Some(node) = doc.get_node(id) else {
            continue;
        };
        stack.extend(node.children.iter().copied());
        let Some(el) = node.element_data() else {
            continue;
        };
        let attr = |name: &str| {
            el.attrs()
                .iter()
                .find(|a| &*a.name.local == name)
                .map(|a| a.value.as_str())
        };
        let base_url = c.base_url.clone();
        match &*el.name.local {
            "img" | "input" => {
                if let Some(src) = attr("src") {
                    c.image_src(src);
                }
            }
            "style" => {
                let text = node.text_content();
                c.css(&text, &base_url, 0);
            }
            "link" => {
                let is_stylesheet = attr("rel").is_some_and(|r| {
                    r.split_ascii_whitespace()
                        .any(|t| t.eq_ignore_ascii_case("stylesheet"))
                });
                if is_stylesheet && let Some(href) = attr("href") {
                    c.stylesheet(href, &base_url, 0);
                }
            }
            _ => {}
        }
        if let Some(style) = attr("style") {
            c.css(style, &base_url, 0);
        }
    }
    c.bundle
}

#[cfg(test)]
mod tests {
    use super::css_urls;

    fn urls(css: &str) -> Vec<(String, bool)> {
        css_urls(css)
            .into_iter()
            .map(|r| (r.url, r.is_import))
            .collect()
    }

    #[test]
    fn finds_quoted_and_unquoted_urls_and_imports() {
        assert_eq!(
            urls(
                r#"a{background:url(a.png)} b{background:URL( "b c.png" )}
                   /* url(skip.png) */ @import 'x.css'; @import url(y.css) print;
                   c{background:url(data:image/png;base64,AAAA)}"#
            ),
            vec![
                ("a.png".into(), false),
                ("b c.png".into(), false),
                ("x.css".into(), true),
                ("y.css".into(), true),
            ]
        );
    }

    #[test]
    fn unterminated_input_does_not_panic() {
        for css in ["url(", "url(\"a", "@import", "/* open", "url('"] {
            let _ = urls(css);
        }
    }
}
