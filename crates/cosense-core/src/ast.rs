#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Heading {
        level: usize,
        text: String,
    },
    Paragraph {
        lines: Vec<String>,
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

#[derive(Debug, Clone, PartialEq)]
pub struct ListItem {
    pub level: usize,
    pub content: ItemContent,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemContent {
    Text(String),
    Code {
        language: String,
        lines: Vec<String>,
    },
    Math {
        lines: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub blocks: Vec<Block>,
}

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
