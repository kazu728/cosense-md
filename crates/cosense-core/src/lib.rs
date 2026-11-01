mod ast;
mod inline;
mod parser;
mod render_markdown;

pub use parser::looks_like_cosense;

pub fn convert(text: &str) -> String {
    render_markdown::render_markdown(&parser::parse(text))
}
