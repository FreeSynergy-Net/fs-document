//! `look.style`, `look.footer`, `[folder_titles]` and `[[menu]]` of `site.toml`.
mod common;
use common::{builds, put, refused, scratch};
use fs_text_site::{build, FolderSource, SiteConfig};
use std::fs;
use std::io::Write;

fn add_toml(t: &tempfile::TempDir, extra: &str) {
    let mut f = fs::OpenOptions::new()
        .append(true)
        .open(t.path().join("site.toml"))
        .unwrap();
    writeln!(f, "{extra}").unwrap();
}

fn built(t: &tempfile::TempDir) -> std::path::PathBuf {
    let out = t.path().join("_out");
    let src = FolderSource::open(t.path()).unwrap();
    build(&src, &out, &common::opts()).expect("builds");
    out
}

fn read(out: &std::path::Path, rel: &str) -> String {
    fs::read_to_string(out.join(rel)).unwrap()
}

const BASE: &str = "schema = 2\npurpose = \"website\"\n[site]\nid = \"s\"\nlanguages = [\"en\"]\n[names]\nprimary = \"example.test\"\n[acme]\nmethod = \"http-01\"\n";

fn parse_err(extra: &str) -> String {
    SiteConfig::parse(&format!("{BASE}{extra}"))
        .expect_err("must be refused")
        .to_string()
}

// ---- E1: user style sheet -------------------------------------------------------------

#[test]
fn style_is_linked_after_the_fixed_sheet_on_every_page() {
    let t = scratch(true);
    put(&t, "files/site.css", b"body{color:red}\n");
    put(&t, "content/en/a.md", b"# A\n");
    add_toml(&t, "[look]\nstyle = \"files/site.css\"");
    let out = built(&t);
    for p in ["en/index.html", "en/a/index.html", "404.html"] {
        let h = read(&out, p);
        let fixed = h.find("/_fs/theme.").expect("fixed sheet");
        let own = h
            .find("href=\"/site.css\"")
            .unwrap_or_else(|| panic!("{p}: no user sheet"));
        assert!(own > fixed, "{p}: the user sheet comes after the fixed one");
    }
    assert_eq!(read(&out, "site.css"), "body{color:red}\n");
}

#[test]
fn without_style_nothing_changes() {
    let t = scratch(true);
    let out = built(&t);
    assert_eq!(
        read(&out, "en/index.html")
            .matches("rel=\"stylesheet\"")
            .count(),
        1
    );
}

#[test]
fn a_style_path_that_leaves_files_is_refused() {
    for bad in [
        "files/../site.toml",
        "/etc/passwd.css",
        "site.css",
        "files//x.css",
        "files/x.txt",
        "files/.h.css",
    ] {
        let e = parse_err(&format!("[look]\nstyle = {bad:?}"));
        assert!(e.contains("look:"), "{bad}: {e}");
    }
}

#[test]
fn a_style_file_that_is_not_there_stops_the_build() {
    let t = scratch(true);
    add_toml(&t, "[look]\nstyle = \"files/none.css\"");
    assert!(refused(&t).contains("look.style: files/none.css is not there"));
}

#[test]
fn a_style_that_is_a_link_out_of_files_stops_the_build() {
    let t = scratch(true);
    put(&t, "outside.css", b"a{}");
    std::os::unix::fs::symlink(
        t.path().join("outside.css"),
        t.path().join("files/site.css"),
    )
    .unwrap();
    add_toml(&t, "[look]\nstyle = \"files/site.css\"");
    let e = refused(&t);
    assert!(e.contains("look.style"), "{e}");
}

// ---- E2: footer -----------------------------------------------------------------------

#[test]
fn footer_is_text_and_escaped() {
    let t = scratch(true);
    add_toml(&t, "[look]\nfooter = \"Imprint & <b>x</b>\"");
    let out = built(&t);
    assert!(read(&out, "en/index.html")
        .contains("<footer><p>Imprint &amp; &lt;b&gt;x&lt;/b&gt;</p></footer>"));
    assert!(read(&out, "404.html").contains("<footer>"));
}

#[test]
fn no_footer_without_the_key() {
    let t = scratch(true);
    assert!(!read(&built(&t), "en/index.html").contains("<footer>"));
}

#[test]
fn a_footer_with_a_line_break_is_refused() {
    assert!(parse_err("[look]\nfooter = \"a\\nb\"").contains("footer"));
}

// ---- E3: folder titles ----------------------------------------------------------------

#[test]
fn a_folder_without_index_gets_its_title_from_site_toml() {
    let t = scratch(true);
    put(&t, "content/en/info/contact.md", b"# Contact\n");
    put(&t, "content/en/other/x.md", b"# X\n");
    add_toml(&t, "[folder_titles]\ninfo = \"Information\"");
    let out = built(&t);
    let h = read(&out, "en/info/index.html");
    assert!(h.contains("<h1>Information</h1>"), "{h}");
    assert!(read(&out, "en/index.html").contains(">Information</a>"));
    // the other folder keeps its name
    assert!(read(&out, "en/index.html").contains(">other</a>"));
}

#[test]
fn a_folder_title_for_no_folder_stops_the_build() {
    let t = scratch(true);
    add_toml(&t, "[folder_titles]\nnone = \"X\"");
    assert!(refused(&t).contains("folder_titles \"none\": there is no such folder"));
}

#[test]
fn a_folder_with_an_index_page_uses_the_title_of_that_page() {
    let t = scratch(true);
    put(&t, "content/en/info/index.md", b"# Real title\n");
    put(&t, "content/en/info/a.md", b"# A\n");
    add_toml(&t, "[folder_titles]\ninfo = \"Ignored\"");
    let out = built(&t);
    assert!(read(&out, "en/index.html").contains(">Real title</a>"));
    assert!(!read(&out, "en/index.html").contains("Ignored"));
}

// ---- E4: menu -------------------------------------------------------------------------

#[test]
fn a_menu_replaces_the_tree_in_the_navigation() {
    let t = scratch(true);
    put(&t, "content/en/info/contact.md", b"# Contact\n");
    put(&t, "content/en/about.md", b"# About\n");
    add_toml(
        &t,
        "[[menu]]\nlabel = \"Home\"\ntarget = \"/\"\n[[menu]]\nlabel = \"Reach us\"\ntarget = \"info/contact\"\n[[menu]]\nlabel = \"Source\"\ntarget = \"https://example.org/src\"",
    );
    let out = built(&t);
    let h = read(&out, "en/about/index.html");
    let nav = h
        .split("<nav class=\"side\"")
        .nth(1)
        .unwrap()
        .split("</nav>")
        .next()
        .unwrap();
    assert!(nav.contains("<a href=\"/en/\">Home</a>"), "{nav}");
    assert!(
        nav.contains("<a href=\"/en/info/contact/\">Reach us</a>"),
        "{nav}"
    );
    assert!(
        nav.contains("<a href=\"https://example.org/src\">Source</a>"),
        "{nav}"
    );
    assert!(!nav.contains(">About</a>"), "the tree is not shown: {nav}");
    // the contents page still lists everything
    assert!(read(&out, "en/contents/index.html").contains(">About</a>"));
}

#[test]
fn a_menu_target_that_does_not_exist_stops_the_build() {
    let t = scratch(true);
    add_toml(&t, "[[menu]]\nlabel = \"Gone\"\ntarget = \"nowhere\"");
    let e = refused(&t);
    assert!(
        e.contains("menu \"Gone\": the page /nowhere does not exist in language en"),
        "{e}"
    );
}

#[test]
fn a_menu_target_that_exists_builds() {
    let t = scratch(true);
    put(&t, "content/en/a.md", b"# A\n");
    add_toml(&t, "[[menu]]\nlabel = \"A\"\ntarget = \"a/\"");
    assert!(builds(&t));
}

#[test]
fn bad_menu_targets_and_labels_are_refused() {
    for t in [
        "http://example.org",
        "javascript:alert(1)",
        "//example.org",
        "https://",
        "A/B",
        "../x",
        "https://a b",
    ] {
        let e = parse_err(&format!("[[menu]]\nlabel = \"L\"\ntarget = {t:?}"));
        assert!(e.contains("target"), "{t}: {e}");
    }
    assert!(parse_err("[[menu]]\nlabel = \"\"\ntarget = \"a\"").contains("label"));
}

#[test]
fn look_needs_schema_2() {
    let e = SiteConfig::parse("schema = 1\npurpose = \"website\"\n[names]\nprimary = \"example.test\"\n[acme]\nmethod = \"http-01\"\n[look]\nfooter = \"x\"\n")
        .unwrap_err()
        .to_string();
    assert!(e.contains("schema = 2"), "{e}");
}

#[test]
fn the_template_is_valid_and_uses_every_new_key() {
    let c =
        SiteConfig::parse(include_str!("fixtures/template/site.toml")).expect("template parses");
    assert_eq!(c.look.style.as_deref(), Some("site.css"));
    assert!(c.look.footer.is_some());
    assert_eq!(c.look.menu.len(), 3);
    assert_eq!(c.look.folder_titles.len(), 1);
}
