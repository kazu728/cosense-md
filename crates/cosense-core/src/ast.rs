//! Target-independent syntax tree for Cosense documents.
//!
//! Blocks mirror the two-phase parser design (tokenizer → match → AST). Inline
//! content is itself an AST: `[...]` spans are classified once into typed nodes
//! so that each output target renders them by walking the tree, rather than by
//! re-running ordered regex substitutions.

#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Heading {
        level: usize,
        text: String,
    },
    Paragraph {
        text: String,
    },
    BlankLine,
    Code {
        language: String,
        lines: Vec<String>,
        indent: String,
    },
    Math {
        lines: Vec<String>,
        indent: String,
    },
    BulletList {
        items: Vec<ListItem>,
    },
    Table {
        title: Option<String>,
        header: Vec<String>,
        rows: Vec<Vec<String>>,
    },
}

/// One bullet: its nesting level (0-based) and inline text. Cosense lists are a
/// flat sequence — irregular indentation is normalized to sequential levels while
/// parsing — so there is no recursive tree to build, render, or drop.
#[derive(Debug, Clone, PartialEq)]
pub struct ListItem {
    pub level: usize,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub blocks: Vec<Block>,
}

/// One classified `[...]` span or plain-text run inside a line.
///
/// `Unknown` carries the original source verbatim; anything the classifier does
/// not recognise passes through untouched, which is how the core guarantees it
/// never drops or corrupts input it cannot interpret.
#[derive(Debug, Clone, PartialEq)]
pub enum Inline {
    Text(String),
    Emphasis(Emphasis),
    Tag(String),
    Image { alt: String, url: String },
    Link { label: String, url: String },
    AutoLink(String),
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Emphasis {
    pub bold: bool,
    pub italic: bool,
    pub strike: bool,
    pub text: String,
}
