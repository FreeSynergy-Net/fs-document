# FreeSynergy Document - one text model, from Markdown to a website

**A building block that turns written text into a document model, into HTML and into a static website.**

This workspace holds three crates. It is not a program and it holds no data. It was built with help from Claude (AI).

| Crate | What it is |
|---|---|
| `fs-document` | The document model (blocks, marks, links, history) with CommonMark import and export. No other FreeSynergy crate. |
| `fs-document-html` | HTML output of the model, the page frame and the safe pass-through of user files. |
| `fs-site` | A folder of text and your own files in, a static website folder out (binary `fs-site`). |

## What it can do today

- Read and write Markdown into one model; export the model as HTML.
- Build a deterministic static site from a folder (several languages, menu, own style sheet, footer).

## What is not there yet

- Not published: no tag, no crates.io release. The repository name is a working name.
- No export from a FreeSynergy text store; the site is built from a folder.

## For developers

- Build and test: `cargo build --workspace` and `cargo test --workspace`.
- Design: see `DESIGN.md` in each crate. Licence: MIT OR Apache-2.0.
