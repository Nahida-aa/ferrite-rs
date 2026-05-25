use std::fmt;

/// Java 对照: net.minecraft.server.packs.repository.KnownPack
///
/// Java 版有一个 `STREAM_CODEC` 静态字段用于网络序列化。
/// Rust 版暂不需要网络序列化，且缺少合适的编码器基础设施，
/// 因此这里不提供 `stream_codec()`。等需要时用
/// `network::codec::stream_codec::composite3` 配合
/// `network::codec::bytebuf_codecs::string_utf8` 构建即可。
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

impl fmt::Display for KnownPack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.namespace, self.id, self.version)
    }
}
