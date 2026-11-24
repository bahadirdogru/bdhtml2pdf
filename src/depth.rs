//! Very deep element nesting.
//!
//! Style, layout and fulgur's conversion recurse once per nesting level, and
//! fulgur skips everything below 512 levels. A page with thousands of nested
//! elements would therefore overflow the stack or lose its deepest text.
//!
//! The HTML parsers in Blink and WebKit stop nesting at 512 levels and attach
//! deeper elements as siblings. [`flatten_deep_nesting`] does the same on the
//! final DOM: everything below [`MAX_NESTING_DEPTH`] moves up to that level, in
//! document order. Text and images are kept; styling that came from the
//! removed nesting (inherited bold, indentation per level) is lost.

use std::ops::DerefMut;

use blitz_dom::{BaseDocument, DocumentConfig};
use blitz_html::HtmlDocument;
use blitz_traits::node_id::NodeId;

/// The deepest level an element can have, counted from `<html>` (level 0).
/// Below fulgur's 512 so that it never cuts content.
pub const MAX_NESTING_DEPTH: usize = 500;

/// Elements whose content is text, not markup, or lives outside the tree.
/// They move as a whole.
fn moves_whole(node: &blitz_dom::Node) -> bool {
    node.element_data().is_some_and(|e| {
        matches!(
            &*e.name.local,
            "script"
                | "style"
                | "title"
                | "textarea"
                | "xmp"
                | "iframe"
                | "noembed"
                | "noframes"
                | "noscript"
                | "plaintext"
                | "template"
        )
    })
}

/// Moves every node below [`MAX_NESTING_DEPTH`] up to that level. Returns
/// the number of nodes moved.
pub fn flatten_deep_nesting(doc: &mut BaseDocument) -> usize {
    // Find the nodes at the depth limit that have children (iteratively; the
    // tree may be far deeper than the stack allows to recurse).
    let mut caps = Vec::new();
    let mut stack = vec![(doc.root_element().id, 0usize)];
    while let Some((id, depth)) = stack.pop() {
        let Some(node) = doc.get_node(id) else {
            continue;
        };
        if node.children.is_empty() || moves_whole(node) {
            continue;
        }
        if depth == MAX_NESTING_DEPTH {
            caps.push(id);
        } else {
            stack.extend(node.children.iter().map(|&c| (c, depth + 1)));
        }
    }

    let mut moved = 0;
    for cap in caps {
        // All descendants of `cap` in document order.
        let mut order = Vec::new();
        let mut stack: Vec<NodeId> = doc
            .get_node(cap)
            .map_or_else(Vec::new, |n| n.children.iter().rev().copied().collect());
        while let Some(id) = stack.pop() {
            order.push(id);
            let Some(node) = doc.get_node(id) else {
                continue;
            };
            if !moves_whole(node) {
                stack.extend(node.children.iter().rev());
            }
        }

        let mut m = doc.mutate();
        // Detach deepest first, so every removal touches a single node and the
        // whole pass stays linear.
        for &id in order.iter().rev() {
            m.remove_node(id);
        }
        m.append_children(cap, &order);
        drop(m);
        moved += order.len();
    }
    moved
}

/// Serializes the document as HTML.
pub fn serialize(doc: &BaseDocument) -> String {
    let mut out = String::from("<!DOCTYPE html>");
    doc.root_element().write_outer_html(&mut out);
    out
}

/// Parses `html` and flattens it if it nests deeper than [`MAX_NESTING_DEPTH`].
/// Returns `html` unchanged otherwise.
pub fn flatten_html(html: String) -> String {
    // A page with fewer tags than the limit cannot nest that deep.
    if html.bytes().filter(|&b| b == b'<').count() <= MAX_NESTING_DEPTH {
        return html;
    }
    let mut doc = HtmlDocument::from_html(&html, DocumentConfig::default());
    if flatten_deep_nesting(doc.deref_mut()) == 0 {
        return html;
    }
    serialize(&doc)
}

/// Stack size for threads that parse, lay out or convert untrusted pages.
pub const LARGE_STACK: usize = 256 << 20;

/// Runs `f` on a thread with a [`LARGE_STACK`] and waits for it.
pub fn with_large_stack<T: Send>(f: impl FnOnce() -> T + Send) -> T {
    std::thread::scope(|s| {
        std::thread::Builder::new()
            .name("render".into())
            .stack_size(LARGE_STACK)
            .spawn_scoped(s, f)
            .expect("spawn render thread")
            .join()
            .unwrap_or_else(|e| std::panic::resume_unwind(e))
    })
}
