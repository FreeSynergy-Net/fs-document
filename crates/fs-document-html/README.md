# fs-document-html

HTML output for the `fs-document` model: an `Exporter` that writes a document as HTML, the
page frame with a short navigation, and a safe pass-through that copies user files byte for
byte after path and media checks. Built with help from Claude (AI).

## Example

```rust
use fs_document::{import_markdown, Exporter};
use fs_document_html::HtmlExporter;

let html = HtmlExporter::default().export(&import_markdown("# Hello\n"));
```

## Limits

- Raw HTML in a document is only passed through for an allow-list of tags; everything else is escaped.
- One fixed theme; a site can link its own style sheet (see `fs-site`).
- Unix only for the byte-exact pass-through (on other systems it refuses clearly).
- Version 0.0.x: the API can still change.

Design: [DESIGN.md](DESIGN.md). Licence: MIT OR Apache-2.0.
