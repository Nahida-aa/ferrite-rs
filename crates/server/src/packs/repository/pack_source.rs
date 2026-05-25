/// Describes where a pack came from and how its name should be displayed.
///
/// No class in the Java source implements `PackSource` — every instance is one
/// of the five well-known constants created via `PackSource.create(...)`.
/// An enum captures the same semantics with native equality.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackSource {
    Default,
    BuiltIn,
    Feature,
    World,
    Server,
}

impl PackSource {
    /// Decorate the pack description for display (replaces `Component` when ported).
    pub fn decorate(&self, description: &str) -> String {
        match self {
            PackSource::Default => description.to_owned(),
            PackSource::BuiltIn => format!("{description} [pack.source.builtin]"),
            PackSource::Feature => format!("{description} [pack.source.feature]"),
            PackSource::World => format!("{description} [pack.source.world]"),
            PackSource::Server => format!("{description} [pack.source.server]"),
        }
    }

    /// Whether this pack should be enabled by default.
    pub fn should_add_automatically(&self) -> bool {
        !matches!(self, PackSource::Feature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_passthrough() {
        assert_eq!(PackSource::Default.decorate("hello"), "hello");
        assert!(PackSource::Default.should_add_automatically());
    }

    #[test]
    fn built_in_has_suffix() {
        let out = PackSource::BuiltIn.decorate("My Pack");
        assert!(out.contains("My Pack"));
        assert!(out.contains("pack.source.builtin"));
        assert!(PackSource::BuiltIn.should_add_automatically());
    }

    #[test]
    fn feature_not_auto() {
        assert!(!PackSource::Feature.should_add_automatically());
    }

    #[test]
    fn equality_matches_java_reference_equality() {
        assert_eq!(PackSource::Feature, PackSource::Feature);
        assert_ne!(PackSource::Feature, PackSource::Default);
    }
}
