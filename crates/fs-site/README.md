# fs-site

A folder of text and your own files in, a static website folder out. Same input, same bytes
out (every file gets one fixed modification time).

## Build a site

Layout of the source folder (a copy of `tests/fixtures/template/site.toml` has every key):

```text
site.toml                 the site: names, languages, look, menu
content/<lang>/index.md   the start page of a language
content/<lang>/<folder>/<page>.md   more pages; images next to them are published when used
files/                    published as they are (CSS, favicon, robots.txt ...)
```

```text
fs-site build <source> <output>          # one site; <output> must not exist yet
fs-site build-all <sites-root> <output-root>   # several sites of one domain
```

`SITE_MTIME` (or `--mtime N`) is the modification time of the output in seconds since 1970.
Without it the fixed value 1577836800 (2020-01-01 UTC) is used, never the clock, so two builds
of one source are byte-identical. A value that is not a number is refused.

## Keys for your own look (schema 2)

- `[look] style = "files/site.css"`: your style sheet, linked after the fixed one on every
  page. The path must lie inside `files/` (no `..`, no link leaving the folder) and end in `.css`.
- `[look] footer = "text"`: one line of plain text in the footer of every page (escaped).
- `[[menu]] label = "..." target = "..."`: the main menu instead of the page tree. The
  target is a page of the site (`info/contact`, `/` for the start page) or a full `https://`
  address. A page that does not exist stops the build. The contents page still lists all pages.
- `[folder_titles] info = "Information"`: the name of a folder without `index.md` in the
  navigation and on its overview page. A folder with `index.md` takes the title of that page.

Every problem is listed at once; if one stops the build, nothing is written.
Menu labels, footer and folder titles are one text for all languages (no per-language text yet).

## About

Part of the `fs-document` workspace; built with help from Claude (AI). Limits: a site is built from a
folder only (no store source yet); byte-exact copying of your own files works on Unix only
(other systems refuse with a clear error). Licence: MIT OR Apache-2.0.
