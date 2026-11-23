//! `[look]`, `[[menu]]` and `[folder_titles]` of `site.toml` (schema 2): the user's own style
//! sheet, footer text, main menu and folder names.
//!
//! Pattern: Builder from a raw, typed form (serde, unknown keys refused) into checked value
//! objects. Whether a menu target or a folder exists is a question for the build (it needs the
//! pages): see `menu.rs`; here only the form is checked.

use crate::error::ConfigError;
use crate::host::name_char_bad;
use crate::walk::url_segment_ok;
use fs_text_html::pass::PassPath;
use serde::Deserialize;
use std::collections::BTreeMap;

const MAX_MENU: usize = 12;
const MAX_LABEL: usize = 40;
const MAX_FOOTER: usize = 300;
const MAX_FOLDER_TITLES: usize = 100;
const MAX_URL: usize = 500;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawLook {
    pub(crate) style: Option<String>,
    pub(crate) footer: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawMenu {
    pub(crate) label: String,
    pub(crate) target: String,
}

/// Where a menu entry leads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MenuTarget {
    /// A page of the site: address segments below the language (empty = the start page).
    Page(Vec<String>),
    /// A complete `https://` address.
    External(String),
}

/// One entry of the main menu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuItem {
    /// Text shown.
    pub label: String,
    /// Where it leads.
    pub target: MenuTarget,
}

/// The choices of the author about the look of the site; all optional.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Look {
    /// `look.style`: a style sheet of the user, as a path below `files/`; linked after the
    /// fixed style sheet on every page.
    pub style: Option<String>,
    /// `look.footer`: plain text of the footer (shown escaped, like every text).
    pub footer: Option<String>,
    /// `[[menu]]`: the main menu; empty = the navigation tree as before.
    pub menu: Vec<MenuItem>,
    /// `[folder_titles]`: the title of a folder that has no `index.md` (key = the folder as
    /// `a/b`).
    pub folder_titles: BTreeMap<Vec<String>, String>,
}

fn err(m: impl Into<String>) -> ConfigError {
    ConfigError::Look(m.into())
}

fn text_ok(s: &str, max: usize) -> bool {
    !s.trim().is_empty() && s.chars().count() <= max && !s.chars().any(name_char_bad)
}

fn style_of(s: &str) -> Result<String, ConfigError> {
    let rest = s
        .strip_prefix("files/")
        .ok_or_else(|| err("style must be a path inside files/, for example \"files/site.css\""))?;
    let p = PassPath::parse(rest).map_err(|e| err(format!("style {s:?}: {e}")))?;
    if p.extension().as_deref() != Some("css") {
        return Err(err(format!("style {s:?} must be a .css file")));
    }
    Ok(p.as_str().to_string())
}

fn target_of(t: &str) -> Result<MenuTarget, ConfigError> {
    if let Some(rest) = t.strip_prefix("https://") {
        let host_ok = rest
            .split(['/', '?', '#'])
            .next()
            .is_some_and(|h| !h.is_empty());
        let clean = !t
            .chars()
            .any(|c| c.is_control() || c.is_whitespace() || c == '\\');
        if !(host_ok && clean && t.len() <= MAX_URL) {
            return Err(err(format!("target {t:?} is not a complete https address")));
        }
        return Ok(MenuTarget::External(t.to_string()));
    }
    let segs: Vec<String> = t
        .trim_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    if segs.iter().all(|s| url_segment_ok(s)) && !t.contains("//") {
        Ok(MenuTarget::Page(segs))
    } else {
        Err(err(format!(
            "target {t:?} is neither a page of the site (a/b, lower case) nor a complete https address"
        )))
    }
}

impl Look {
    pub(crate) fn from_raw(
        look: Option<RawLook>,
        menu: Option<Vec<RawMenu>>,
        titles: Option<BTreeMap<String, String>>,
    ) -> Result<Self, ConfigError> {
        let mut out = Self::default();
        if let Some(l) = look {
            out.style = l.style.as_deref().map(style_of).transpose()?;
            if let Some(f) = l.footer {
                if !text_ok(&f, MAX_FOOTER) {
                    return Err(err(format!(
                        "footer: 1 to {MAX_FOOTER} characters on one line, no control characters"
                    )));
                }
                out.footer = Some(f);
            }
        }
        let menu = menu.unwrap_or_default();
        if menu.len() > MAX_MENU {
            return Err(err(format!("menu: more than {MAX_MENU} entries")));
        }
        for m in menu {
            if !text_ok(&m.label, MAX_LABEL) {
                return Err(err(format!(
                    "menu label {:?}: 1 to {MAX_LABEL} characters, no control characters",
                    m.label
                )));
            }
            out.menu.push(MenuItem {
                target: target_of(&m.target)?,
                label: m.label,
            });
        }
        let titles = titles.unwrap_or_default();
        if titles.len() > MAX_FOLDER_TITLES {
            return Err(err("folder_titles: too many entries"));
        }
        for (k, v) in titles {
            let segs: Vec<String> = k.split('/').map(str::to_string).collect();
            if !segs.iter().all(|s| url_segment_ok(s)) {
                return Err(err(format!(
                    "folder_titles key {k:?} is not a folder (a/b)"
                )));
            }
            if !text_ok(&v, MAX_LABEL * 2) {
                return Err(err(format!("folder_titles {k:?}: 1 to 80 characters")));
            }
            out.folder_titles.insert(segs, v);
        }
        Ok(out)
    }
}
