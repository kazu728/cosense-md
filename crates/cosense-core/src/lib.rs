//! Cosense → Markdown conversion core.
//!
//! `convert` is a total, panic-free `&str -> String` function: input-dependent
//! paths never `unwrap`, `panic!`, or index by a computed offset, so any input
//! produces some output rather than aborting.

mod ast;
mod inline;
mod parser;
mod render_markdown;

pub use ast::{Block, Document, Emphasis, Inline, ListItem};
pub use inline::IMAGE_EXTENSIONS;
pub use parser::looks_like_cosense;

pub fn convert(text: &str) -> String {
    render_markdown::render_markdown(&parser::parse(text))
}
