use std::fmt;

/// Java 对照: net.minecraft.server.packs.repository.KnownPack
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct KnownPack {
    pub namespace: String,
    pub id: String,
    pub version: String,
}

impl KnownPack {
    pub const VANILLA_NAMESPACE: &'static str = "minecraft";
    pub const GAME_VERSION: &'static str = "26.1.2";

    pub fn vanilla(id: impl Into<String>) -> Self {
        Self {
            namespace: Self::VANILLA_NAMESPACE.to_owned(),
            id: id.into(),
            version: Self::GAME_VERSION.to_owned(),
        }
    }

    pub fn is_vanilla(&self) -> bool {
        self.namespace == Self::VANILLA_NAMESPACE
    }
}
// == java o.toString()
impl fmt::Display for KnownPack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.namespace, self.id, self.version)
    }
}
