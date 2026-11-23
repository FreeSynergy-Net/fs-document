//! Example: print the HTML fragment of the Markdown file given as argument.
//!
//! Pattern: Facade usage — only the crate root API.
use fs_document_html::HtmlExporter;
use fs_document::{import_markdown, Exporter};

fn main() {
    let path = std::env::args().nth(1).expect("a Markdown file");
    let src = std::fs::read_to_string(path).expect("readable file");
    print!("{}", HtmlExporter::default().export(&import_markdown(&src)));
}
