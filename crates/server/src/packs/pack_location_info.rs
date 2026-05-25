use crate::packs::repository::known_pack::KnownPack;
use crate::packs::repository::pack_source::PackSource;

/// Identifying information for a pack.
pub struct PackLocationInfo {
    pub id: String,
    pub title: String, // Component when ported
    pub source: PackSource,
    pub known_pack_info: Option<KnownPack>,
}

impl PackLocationInfo {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        source: PackSource,
        known_pack_info: Option<KnownPack>,
    ) -> Self {
        Self { id: id.into(), title: title.into(), source, known_pack_info }
    }

    /// Creates a chat link component for this pack.
    /// Stub — awaits `Component` port.
    pub fn create_chat_link(&self, _enabled: bool, _description: &str) -> String {
        // When Component is ported, this should produce a hoverable
        // bracketed link similar to the Java original.
        format!("[{}]", self.source.decorate(&self.id))
    }
}

impl Default for PackLocationInfo {
    fn default() -> Self {
        Self {
            id: String::new(),
            title: String::new(),
            source: PackSource::Default,
            known_pack_info: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_location() {
        let info = PackLocationInfo::new("my_pack", "My Pack", PackSource::Default, None);
        assert_eq!(info.id, "my_pack");
        assert_eq!(info.title, "My Pack");
        assert!(info.known_pack_info.is_none());
    }

    #[test]
    fn chat_link_uses_source() {
        let info = PackLocationInfo::new("test", "Test", PackSource::BuiltIn, None);
        let link = info.create_chat_link(true, "description");
        assert!(link.contains("test"));
    }
}
