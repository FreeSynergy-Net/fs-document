//! Windows counterpart of `pass_fs.rs`: the pass-through is Unix only, so it must refuse, not copy.
#![cfg(not(unix))]
use fs_document_html::pass::{copy_file, Limits, PassError, PassPath};

#[test]
fn pass_through_refuses_on_non_unix_and_writes_nothing() {
    let (root, dest) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    std::fs::write(root.path().join("a.html"), "<p>a</p>").unwrap();
    let r = copy_file(
        root.path(),
        &PassPath::parse("a.html").unwrap(),
        dest.path(),
        Limits::default(),
    );
    assert!(matches!(r, Err(PassError::NotRegular(_))), "{r:?}");
    assert!(std::fs::read_dir(dest.path()).unwrap().next().is_none());
}
