//! The main menu, the titles of folders and the user's style sheet at build time.
//!
//! Pattern: Specification over the pages that exist (`check`, phase 1: a target that is not
//! there stops the build before anything is written) and a Strategy for the navigation of a
//! language (`Writer::navigation`: the tree as before, or the menu of `site.toml`).

use crate::config::{Lang, SiteConfig};
use crate::index::{page_url, SiteIndex};
use crate::look::MenuTarget;
use crate::report::BuildReport;
use crate::source::FileEntry;
use crate::tree::{build as build_tree, TreePage};
use crate::write::Writer;
use fs_text_html::{NavNode, Navigation};

/// Check what `site.toml` names against what the site has. Every problem is listed.
pub(crate) fn check(
    cfg: &SiteConfig,
    index: &SiteIndex,
    passed: &[(FileEntry, [u8; 32], bool)],
    rep: &mut BuildReport,
) {
    let look = &cfg.look;
    if let Some(s) = &look.style {
        if !passed.iter().any(|(f, _, _)| f.path.as_str() == s) {
            rep.note(
                "site.toml",
                format!("look.style: files/{s} is not there (or was refused above)"),
                true,
            );
        }
    }
    // languages that have pages: a menu entry must lead somewhere in each of them
    let langs: Vec<&Lang> = cfg
        .languages
        .iter()
        .filter(|l| index.pages.keys().any(|(k, _)| k == *l))
        .collect();
    for m in &look.menu {
        if let MenuTarget::Page(p) = &m.target {
            for l in &langs {
                if !index.pages.contains_key(&((*l).clone(), p.clone())) {
                    rep.note(
                        "site.toml",
                        format!(
                            "menu {:?}: the page /{} does not exist in language {}",
                            m.label,
                            p.join("/"),
                            l.as_str()
                        ),
                        true,
                    );
                }
            }
        }
    }
    for k in look.folder_titles.keys() {
        let shown = k.join("/");
        let kinds: Vec<usize> = langs
            .iter()
            .filter_map(|l| index.pages.get(&((*l).clone(), k.clone())).copied())
            .collect();
        if kinds.is_empty() {
            rep.note(
                "site.toml",
                format!("folder_titles {shown:?}: there is no such folder"),
                true,
            );
        } else if kinds.iter().any(|i| *i != usize::MAX) {
            rep.note(
                "site.toml",
                format!(
                    "folder_titles {shown:?}: the folder has an index page, whose title is used"
                ),
                false,
            );
        }
    }
}

impl Writer<'_> {
    /// The title of a folder without an index page: `[folder_titles]`, else its name.
    pub(crate) fn folder_title(&self, f: &[String]) -> String {
        self.cfg
            .look
            .folder_titles
            .get(f)
            .cloned()
            .unwrap_or_else(|| f.last().cloned().unwrap_or_default())
    }

    /// The navigation of `lang`: the menu of `site.toml` if it has one, else the tree.
    pub(crate) fn navigation(
        &self,
        lang: &Lang,
        pages: &[TreePage],
        extra: Vec<NavNode>,
        more: &str,
    ) -> Navigation {
        let menu = &self.cfg.look.menu;
        if menu.is_empty() {
            return build_tree(&self.prefix, lang, pages, extra, more);
        }
        let roots = menu
            .iter()
            .map(|m| NavNode {
                title: m.label.clone(),
                href: match &m.target {
                    MenuTarget::Page(p) => page_url(&self.prefix, lang, p),
                    MenuTarget::External(u) => u.clone(),
                },
                children: Vec::new(),
            })
            .collect();
        Navigation {
            roots,
            extra,
            more: more.to_string(),
        }
    }
}
