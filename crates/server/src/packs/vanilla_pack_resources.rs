use crate::packs::abstract_pack_resources::PackResourcesCommon;

/// The built-in vanilla pack.
///
/// Java 对照: net.minecraft.server.packs.VanillaPackResources
pub struct VanillaPackResources {
    pub common: PackResourcesCommon,
    // TODO: namespace list, root paths, paths for each type
}
