# fs-document

The document model of FreeSynergy as a plain library: text as a rope with stand-off mark
layers, blocks, links, undo history, and a CommonMark import and export. No store, no GUI,
no server, no other FreeSynergy crate. Built with help from Claude (AI).

## Example

```rust
use fs_document::{export_markdown, import_markdown, ExportOptions};

let doc = import_markdown("# Title\n\nSome *text* with a [link](other.md).\n");
let markdown = export_markdown(&doc, ExportOptions::default());
```

## Limits

- Extensions beyond CommonMark (layers, admonitions, templates) are our own; they are not a standard.
- `ExportOptions::layers` writes layer marks with attribute syntax that other tools do not read.
- Version 0.0.x: the API can still change between versions.

Design: [DESIGN.md](DESIGN.md). Licence: MIT OR Apache-2.0.
