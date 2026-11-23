//! Windows only: what `open_plain` must refuse there, one test per check. The links are made on
//! the runner with `mklink` (junction `/J`, directory link `/D`, file link): the same system
//! calls an attacker would use.
#![cfg(windows)]
use fs_document_html::pass::{copy_file, Copied, Limits, PassError, PassPath};
use std::path::Path;
use std::process::Command;

fn copy(root: &Path, dest: &Path, rel: &str) -> Result<Copied, PassError> {
    copy_file(root, &PassPath::parse(rel)?, dest, Limits::default())
}

fn mklink(args: &[&str], link: &Path, target: &Path) {
    let st = Command::new("cmd")
        .arg("/C")
        .arg("mklink")
        .args(args)
        .arg(link)
        .arg(target)
        .output()
        .unwrap();
    assert!(st.status.success(), "mklink {args:?}: {st:?}");
}

/// A `root` with a `real.txt`, an `outside` folder with `secret.txt`, an empty `dest`.
fn setup() -> (tempfile::TempDir, tempfile::TempDir, tempfile::TempDir) {
    let (root, outside, dest) = (
        tempfile::tempdir().unwrap(),
        tempfile::tempdir().unwrap(),
        tempfile::tempdir().unwrap(),
    );
    std::fs::write(root.path().join("real.txt"), "real").unwrap();
    std::fs::write(outside.path().join("secret.txt"), "secret").unwrap();
    (root, outside, dest)
}

fn refused(r: Result<Copied, PassError>, what: &str, dest: &Path) {
    assert!(matches!(r, Err(PassError::NotRegular(_))), "{what}: {r:?}");
    assert!(
        std::fs::read_dir(dest).unwrap().next().is_none(),
        "{what}: nothing was written"
    );
}

#[test]
fn a_plain_file_below_a_plain_folder_is_copied() {
    let (root, _o, dest) = setup();
    std::fs::create_dir(root.path().join("sub")).unwrap();
    std::fs::write(root.path().join("sub").join("a.txt"), "a").unwrap();
    assert!(copy(root.path(), dest.path(), "sub/a.txt").is_ok());
    assert!(copy(root.path(), dest.path(), "real.txt").is_ok());
}

#[test]
fn a_file_symbolic_link_is_refused() {
    let (root, o, dest) = setup();
    mklink(
        &[],
        &root.path().join("link.txt"),
        &o.path().join("secret.txt"),
    );
    mklink(
        &[],
        &root.path().join("inner.txt"),
        &root.path().join("real.txt"),
    );
    for rel in ["link.txt", "inner.txt"] {
        refused(copy(root.path(), dest.path(), rel), rel, dest.path());
    }
}

#[test]
fn a_junction_on_the_way_is_refused() {
    let (root, o, dest) = setup();
    mklink(&["/J"], &root.path().join("dir"), o.path());
    refused(
        copy(root.path(), dest.path(), "dir/secret.txt"),
        "junction",
        dest.path(),
    );
}

#[test]
fn a_directory_symbolic_link_on_the_way_is_refused() {
    let (root, o, dest) = setup();
    mklink(&["/D"], &root.path().join("dir"), o.path());
    refused(
        copy(root.path(), dest.path(), "dir/secret.txt"),
        "directory link",
        dest.path(),
    );
}

#[test]
fn a_link_deeper_in_the_way_is_refused() {
    let (root, o, dest) = setup();
    std::fs::create_dir(root.path().join("a")).unwrap();
    mklink(&["/J"], &root.path().join("a").join("b"), o.path());
    refused(
        copy(root.path(), dest.path(), "a/b/secret.txt"),
        "deep junction",
        dest.path(),
    );
}

#[test]
fn a_part_of_the_way_that_is_a_file_is_refused() {
    let (root, _o, dest) = setup();
    refused(
        copy(root.path(), dest.path(), "real.txt/x.txt"),
        "file as folder",
        dest.path(),
    );
}

#[test]
fn a_hard_link_is_refused() {
    let (root, _o, dest) = setup();
    std::fs::hard_link(root.path().join("real.txt"), root.path().join("twin.txt")).unwrap();
    for rel in ["real.txt", "twin.txt"] {
        refused(copy(root.path(), dest.path(), rel), rel, dest.path());
    }
}

#[test]
fn a_device_name_is_not_a_file_of_the_folder() {
    // `NUL`, `CON`, `AUX` are devices on Windows whatever the extension; they must not open.
    let (root, _o, dest) = setup();
    for rel in ["nul.txt", "con.txt", "aux.png", "nul", "com1.txt"] {
        assert!(copy(root.path(), dest.path(), rel).is_err(), "{rel}");
    }
    assert!(std::fs::read_dir(dest.path()).unwrap().next().is_none());
}

#[test]
fn a_drive_or_a_backslash_cannot_leave_the_folder() {
    let (root, o, dest) = setup();
    let abs = o.path().join("secret.txt");
    for rel in [
        abs.to_string_lossy().replace('\\', "/"),
        abs.to_string_lossy().to_string(),
        "..\\secret.txt".to_string(),
        "C:/Windows/win.ini".to_string(),
        "real.txt:stream".to_string(),
    ] {
        assert!(copy(root.path(), dest.path(), &rel).is_err(), "{rel}");
    }
}
