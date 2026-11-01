//! Cosense → Markdown conversion core.
//!
//! `convert` is a total, panic-free `&str -> String` function: input-dependent
//! paths never `unwrap` or `panic!`, and slice text only at the boundaries of
//! ASCII markers or `char`s, so any input produces some output rather than
//! aborting. The property tests and the shared fixture guard this.

mod ast;
mod inline;
mod parser;
mod render_markdown;

pub use parser::looks_like_cosense;

pub fn convert(text: &str) -> String {
    render_markdown::render_markdown(&parser::parse(text))
}
