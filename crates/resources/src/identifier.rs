use serde::{Deserialize, Serialize};

/// A resource location consisting of a namespace and a path, e.g. `minecraft:stone`.
/// Both are restricted to lowercase alphanumeric characters plus `._-` for namespaces
/// and `/._-` for paths.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Identifier {
    namespace: String,
    path: String,
}

impl Identifier {
    pub const NAMESPACE_SEPARATOR: char = ':';
    pub const DEFAULT_NAMESPACE: &'static str = "minecraft";
    pub const REALMS_NAMESPACE: &'static str = "realms";

    pub fn from_namespace_and_path(namespace: impl Into<String>, path: impl Into<String>) -> Self {
        let namespace = namespace.into();
        let path = path.into();
        assert_valid_namespace(&namespace, &path);
        assert_valid_path(&namespace, &path);
        Self { namespace, path }
    }

    /// Parses a string of the form `namespace:path`. If no separator is found,
    /// the namespace defaults to `minecraft`.
    pub fn parse(identifier: &str) -> Self {
        Self::by_separator(identifier, Self::NAMESPACE_SEPARATOR)
    }

    pub fn try_parse(identifier: &str) -> Option<Self> {
        Self::try_by_separator(identifier, Self::NAMESPACE_SEPARATOR)
    }

    pub fn with_default_path(path: impl Into<String>) -> Self {
        let path = path.into();
        assert_valid_path(Self::DEFAULT_NAMESPACE, &path);
        Self {
            namespace: Self::DEFAULT_NAMESPACE.to_owned(),
            path,
        }
    }

    pub fn try_build(namespace: &str, path: &str) -> Option<Self> {
        if Self::is_valid_namespace(namespace) && Self::is_valid_path(path) {
            Some(Self { namespace: namespace.to_owned(), path: path.to_owned() })
        } else {
            None
        }
    }

    pub fn by_separator(identifier: &str, separator: char) -> Self {
        if let Some(sep_index) = identifier.find(separator) {
            let path = &identifier[sep_index + 1..];
            if sep_index != 0 {
                let namespace = &identifier[..sep_index];
                Self::create_untrusted(namespace, path)
            } else {
                Self::with_default_path(path)
            }
        } else {
            Self::with_default_path(identifier)
        }
    }

    pub fn try_by_separator(identifier: &str, separator: char) -> Option<Self> {
        if let Some(sep_index) = identifier.find(separator) {
            let path = &identifier[sep_index + 1..];
            if !Self::is_valid_path(path) {
                return None;
            }
            if sep_index != 0 {
                let namespace = &identifier[..sep_index];
                if Self::is_valid_namespace(namespace) {
                    Some(Self { namespace: namespace.to_owned(), path: path.to_owned() })
                } else {
                    None
                }
            } else {
                Some(Self { namespace: Self::DEFAULT_NAMESPACE.to_owned(), path: path.to_owned() })
            }
        } else if Self::is_valid_path(identifier) {
            Some(Self { namespace: Self::DEFAULT_NAMESPACE.to_owned(), path: identifier.to_owned() })
        } else {
            None
        }
    }

    fn create_untrusted(namespace: &str, path: &str) -> Self {
        Self {
            namespace: assert_valid_namespace(namespace, path).to_owned(),
            path: assert_valid_path(namespace, path).to_owned(),
        }
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn with_path(&self, new_path: impl Into<String>) -> Self {
        let new_path = new_path.into();
        assert_valid_path(&self.namespace, &new_path);
        Self { namespace: self.namespace.clone(), path: new_path }
    }

    pub fn map_path(&self, f: impl FnOnce(&str) -> String) -> Self {
        self.with_path(f(&self.path))
    }

    pub fn with_prefix(&self, prefix: &str) -> Self {
        self.with_path(format!("{}{}", prefix, self.path))
    }

    pub fn with_suffix(&self, suffix: &str) -> Self {
        self.with_path(format!("{}{}", self.path, suffix))
    }

    pub fn to_debug_file_name(&self) -> String {
        self.to_string().replace('/', "_").replace(':', "_")
    }

    pub fn to_language_key(&self) -> String {
        format!("{}.{}", self.namespace, self.path)
    }

    pub fn to_short_language_key(&self) -> String {
        if self.namespace == Self::DEFAULT_NAMESPACE {
            self.path.clone()
        } else {
            self.to_language_key()
        }
    }

    pub fn to_short_string(&self) -> String {
        if self.namespace == Self::DEFAULT_NAMESPACE {
            self.path.clone()
        } else {
            self.to_string()
        }
    }

    pub fn to_language_key_with_prefix(&self, prefix: &str) -> String {
        format!("{}.{}", prefix, self.to_language_key())
    }

    pub fn to_language_key_with_prefix_suffix(&self, prefix: &str, suffix: &str) -> String {
        format!("{}.{}.{}", prefix, self.to_language_key(), suffix)
    }

    // ------------------------------------------------------------------
    // Validation
    // ------------------------------------------------------------------

    pub fn is_valid_namespace(namespace: &str) -> bool {
        if namespace == ".." {
            return false;
        }
        namespace.chars().all(Self::valid_namespace_char)
    }

    pub fn is_valid_path(path: &str) -> bool {
        path.chars().all(Self::valid_path_char)
    }

    pub fn is_allowed_in_identifier(c: char) -> bool {
        matches!(c, '0'..='9' | 'a'..='z' | '_' | ':' | '/' | '.' | '-')
    }

    fn valid_namespace_char(c: char) -> bool {
        matches!(c, 'a'..='z' | '0'..='9' | '_' | '-' | '.')
    }

    fn valid_path_char(c: char) -> bool {
        matches!(c, 'a'..='z' | '0'..='9' | '_' | '-' | '/' | '.')
    }
}

// ------------------------------------------------------------------
// Trait impls
// ------------------------------------------------------------------

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

impl std::str::FromStr for Identifier {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_parse(s).ok_or_else(|| format!("invalid identifier: {s}"))
    }
}

impl Ord for Identifier {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.path
            .cmp(&other.path)
            .then_with(|| self.namespace.cmp(&other.namespace))
    }
}

impl PartialOrd for Identifier {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// ------------------------------------------------------------------
// Helpers (panicking validation matching the Java asserts)
// ------------------------------------------------------------------

fn assert_valid_namespace<'a>(namespace: &'a str, path: &str) -> &'a str {
    assert!(
        Identifier::is_valid_namespace(namespace),
        "Non [a-z0-9_.-] character in namespace of identifier: {namespace}:{path}"
    );
    namespace
}

fn assert_valid_path<'a>(namespace: &str, path: &'a str) -> &'a str {
    assert!(
        Identifier::is_valid_path(path),
        "Non [a-z0-9/._-] character in path of location: {namespace}:{path}"
    );
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_with_namespace() {
        let id = Identifier::parse("minecraft:stone");
        assert_eq!(id.namespace(), "minecraft");
        assert_eq!(id.path(), "stone");
    }

    #[test]
    fn parse_default_namespace() {
        let id = Identifier::parse("stone");
        assert_eq!(id.namespace(), "minecraft");
        assert_eq!(id.path(), "stone");
    }

    #[test]
    fn parse_empty_namespace() {
        let id = Identifier::parse(":stone");
        assert_eq!(id.namespace(), "minecraft");
        assert_eq!(id.path(), "stone");
    }

    #[test]
    fn try_parse_valid() {
        assert!(Identifier::try_parse("minecraft:diamond").is_some());
        assert!(Identifier::try_parse("diamond").is_some());
    }

    #[test]
    fn try_parse_invalid_uppercase() {
        assert!(Identifier::try_parse("Stone").is_none());
    }

    #[test]
    fn display() {
        let id = Identifier::parse("minecraft:stone");
        assert_eq!(id.to_string(), "minecraft:stone");
    }

    #[test]
    fn ord_path_first() {
        let a = Identifier::parse("a:z");
        let b = Identifier::parse("b:a");
        assert!(b < a); // "a" < "z" in path
    }

    #[test]
    fn ord_namespace_tiebreak() {
        let a = Identifier::parse("a:stone");
        let b = Identifier::parse("b:stone");
        assert!(a < b);
    }

    #[test]
    fn prefix_suffix() {
        let id = Identifier::parse("minecraft:stone");
        assert_eq!(id.with_prefix("smooth_").path(), "smooth_stone");
        assert_eq!(id.with_suffix("_block").path(), "stone_block");
    }

    #[test]
    fn to_short_string() {
        assert_eq!(Identifier::parse("minecraft:stone").to_short_string(), "stone");
        assert_eq!(Identifier::parse("mod:stone").to_short_string(), "mod:stone");
    }
}
