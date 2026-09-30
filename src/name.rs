//! Naming rules: directories are named `<id>` or `<id>-<label>`, and users refer to them by
//! id, by label, or as `.` for the one they are in.

use std::fmt;
use std::str::FromStr;

/// Stable numeric identifier of a tracked directory.
pub type Id = u32;

const MAX_LABEL_CHARS: usize = 64;

/// Why a label or reference was rejected.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NameError {
    #[error("labels cannot be empty")]
    Empty,
    #[error("labels are limited to {MAX_LABEL_CHARS} characters")]
    TooLong,
    #[error("labels cannot start with '-' or '.'")]
    BadStart,
    #[error("labels cannot be only digits, as numbers refer to ids")]
    Numeric,
    #[error(
        "labels may only contain letters, digits, '-', '_' and '.'{}",
        .suggestion.as_ref().map(|label| format!(" (try '{label}')")).unwrap_or_default()
    )]
    BadChar { suggestion: Option<Label> },
    #[error("expected an id, a label or '.', not a path")]
    Path,
    #[error("ids must fit in 32 bits")]
    IdTooLarge,
}

/// A validated directory label, safe to embed in a file name and to type in a shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label(String);

impl Label {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Turns arbitrary text into a valid label, if possible: each run of disallowed
    /// characters becomes a single `-`.
    fn suggest(text: &str) -> Option<Self> {
        let mut slug = String::new();
        for c in text.chars() {
            if is_label_char(c) {
                slug.push(c);
            } else if !slug.ends_with('-') {
                slug.push('-');
            }
        }
        let slug = slug.trim_start_matches(['-', '.']).trim_end_matches('-');
        slug.chars()
            .take(MAX_LABEL_CHARS)
            .collect::<String>()
            .parse()
            .ok()
    }
}

impl FromStr for Label {
    type Err = NameError;

    fn from_str(s: &str) -> Result<Self, NameError> {
        if s.is_empty() {
            return Err(NameError::Empty);
        }
        if s.chars().count() > MAX_LABEL_CHARS {
            return Err(NameError::TooLong);
        }
        if s.starts_with(['-', '.']) {
            return Err(NameError::BadStart);
        }
        if is_ascii_number(s) {
            return Err(NameError::Numeric);
        }
        if !s.chars().all(is_label_char) {
            return Err(NameError::BadChar {
                suggestion: Self::suggest(s),
            });
        }
        Ok(Self(s.to_owned()))
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// How the user designates a tracked directory: a number is an id, `.` is the one containing
/// the working directory, anything else is a label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirRef {
    Id(Id),
    Label(Label),
    Current,
}

impl FromStr for DirRef {
    type Err = NameError;

    fn from_str(s: &str) -> Result<Self, NameError> {
        if s == "." {
            Ok(Self::Current)
        } else if s.contains('/') {
            Err(NameError::Path)
        } else if is_ascii_number(s) {
            s.parse().map(Self::Id).map_err(|_| NameError::IdTooLarge)
        } else {
            s.parse().map(Self::Label)
        }
    }
}

impl fmt::Display for DirRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Id(id) => write!(f, "id {id}"),
            Self::Label(label) => write!(f, "label '{label}'"),
            Self::Current => f.write_str("the current directory"),
        }
    }
}

/// The name of a tracked directory inside the tempit root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirName {
    pub id: Id,
    pub label: Option<Label>,
}

impl DirName {
    /// Parses a directory name, returning `None` for names tempit did not create.
    pub fn parse(name: &str) -> Option<Self> {
        let (id, label) = match name.split_once('-') {
            Some((id, label)) => (id, Some(label.parse().ok()?)),
            None => (name, None),
        };
        // Require the canonical spelling so that `3` and `03` cannot both claim id 3.
        let id: Id = id.parse().ok().filter(|n: &Id| n.to_string() == id)?;
        Some(Self { id, label })
    }

    /// Whether `reference` designates this directory by id or label. [`DirRef::Current`]
    /// depends on the working directory, not on names, so it never matches here.
    pub fn matches(&self, reference: &DirRef) -> bool {
        match reference {
            DirRef::Id(id) => self.id == *id,
            DirRef::Label(label) => self.label.as_ref() == Some(label),
            DirRef::Current => false,
        }
    }
}

impl fmt::Display for DirName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.label {
            Some(label) => write!(f, "{}-{label}", self.id),
            None => write!(f, "{}", self.id),
        }
    }
}

fn is_label_char(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '-' | '_' | '.')
}

fn is_ascii_number(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn label(s: &str) -> Label {
        s.parse().unwrap()
    }

    fn bad_char(suggestion: Option<&str>) -> NameError {
        NameError::BadChar {
            suggestion: suggestion.map(label),
        }
    }

    #[test]
    fn accepts_reasonable_labels() {
        for ok in ["bugfix", "rust-test", "v1.2", "a_b", "3d", "été", "x"] {
            assert_eq!(ok.parse::<Label>().unwrap().as_str(), ok);
        }
        assert!("a".repeat(MAX_LABEL_CHARS).parse::<Label>().is_ok());
    }

    #[test]
    fn rejects_bad_labels_with_a_suggestion_when_possible() {
        let cases = [
            ("", NameError::Empty),
            ("-x", NameError::BadStart),
            (".hidden", NameError::BadStart),
            ("42", NameError::Numeric),
            ("my project", bad_char(Some("my-project"))),
            ("a/b", bad_char(Some("a-b"))),
            ("a: b", bad_char(Some("a-b"))),
            ("$(rm)", bad_char(Some("rm"))),
            ("   ", bad_char(None)),
            ("!!", bad_char(None)),
        ];
        for (input, expected) in cases {
            assert_eq!(input.parse::<Label>(), Err(expected), "{input:?}");
        }
        let long = "a".repeat(MAX_LABEL_CHARS + 1);
        assert_eq!(long.parse::<Label>(), Err(NameError::TooLong));
    }

    #[test]
    fn suggestions_appear_in_the_message() {
        let err = "my project".parse::<Label>().unwrap_err();
        assert_eq!(
            err.to_string(),
            "labels may only contain letters, digits, '-', '_' and '.' (try 'my-project')"
        );
    }

    #[test]
    fn references_are_ids_labels_or_the_current_directory() {
        assert_eq!("7".parse(), Ok(DirRef::Id(7)));
        assert_eq!("007".parse(), Ok(DirRef::Id(7)));
        assert_eq!(".".parse(), Ok(DirRef::Current));
        assert_eq!("bugfix".parse(), Ok(DirRef::Label(label("bugfix"))));
        assert_eq!("7a".parse(), Ok(DirRef::Label(label("7a"))));
        assert_eq!("99999999999".parse::<DirRef>(), Err(NameError::IdTooLarge));
        assert_eq!("+7".parse::<DirRef>(), Err(bad_char(None)));
        assert_eq!("~/projects".parse::<DirRef>(), Err(NameError::Path));
        assert_eq!("./x".parse::<DirRef>(), Err(NameError::Path));
    }

    #[test]
    fn dir_names_round_trip() {
        for name in ["1", "12", "3-bugfix", "4-with-dashes", "5-7a"] {
            assert_eq!(DirName::parse(name).unwrap().to_string(), name);
        }
        let parsed = DirName::parse("4-with-dashes").unwrap();
        assert_eq!(parsed.id, 4);
        assert_eq!(parsed.label, Some(label("with-dashes")));
    }

    #[test]
    fn foreign_names_are_ignored() {
        for name in [
            "",
            ".lock",
            "03",
            "+3",
            "-3",
            "3-",
            "3-.x",
            "3-a b",
            "x-3",
            "tempit_ab12",
        ] {
            assert_eq!(DirName::parse(name), None, "{name:?}");
        }
    }

    #[test]
    fn matching_by_id_or_label() {
        let name = DirName::parse("3-bugfix").unwrap();
        assert!(name.matches(&DirRef::Id(3)));
        assert!(name.matches(&DirRef::Label(label("bugfix"))));
        assert!(!name.matches(&DirRef::Id(4)));
        assert!(!name.matches(&DirRef::Label(label("bug"))));
        assert!(!name.matches(&DirRef::Current));
    }
}
