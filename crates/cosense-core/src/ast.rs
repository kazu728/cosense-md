#[derive(Debug)]
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
    },
    Math {
        lines: Vec<String>,
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

#[derive(Debug)]
pub struct ListItem {
    pub level: usize,
    pub content: ItemContent,
}

#[derive(Debug)]
pub enum ItemContent {
    Text(String),
    Block(Block),
}

#[derive(Debug)]
pub enum Inline {
    Text(String),
    Emphasis(Emphasis),
    Tag(String),
    Image { alt: String, url: String },
    Link { label: String, url: String },
    AutoLink(String),
}

#[derive(Debug)]
pub struct Emphasis {
    pub bold: bool,
    pub italic: bool,
    pub strike: bool,
    pub text: String,
}
