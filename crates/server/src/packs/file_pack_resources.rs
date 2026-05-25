use crate::packs::abstract_pack_resources::PackResourcesCommon;

/// A pack backed by a zip file.
///
/// Java 对照: net.minecraft.server.packs.FilePackResources
pub struct FilePackResources {
    pub common: PackResourcesCommon,
    // TODO: zip file access
}
