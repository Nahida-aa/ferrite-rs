use std::io;

use resources::identifier::Identifier;
use serde::de::DeserializeOwned;

use crate::packs::abstract_pack_resources::PackResourcesCommon;
use crate::packs::file_pack_resources::FilePackResources;
use crate::packs::metadata::metadata_section_type::MetadataSectionType;
use crate::packs::pack_location_info::PackLocationInfo;
use crate::packs::pack_type::PackType;
use crate::packs::path_pack_resources::PathPackResources;
use crate::packs::repository::known_pack::KnownPack;
use crate::packs::resources::io_supplier::IoSupplier;
use crate::packs::vanilla_pack_resources::VanillaPackResources;

/// Java 对照: net.minecraft.server.packs.PackResources.ResourceOutput
pub trait ResourceOutput: FnMut(Identifier, IoSupplier<Vec<u8>>) {}

impl<T: FnMut(Identifier, IoSupplier<Vec<u8>>)> ResourceOutput for T {}

/// ── PackResources enum (Java 对照: PackResources interface) ──

/// A readable resource pack.
///
/// The set of concrete pack kinds is intentionally closed so that
/// callers never see a bare trait object.  The `Composite` variant
/// has the same semantics as Java's `CompositePackResources`.
pub enum PackResources {
    Path(PathPackResources),
    File(FilePackResources),
    Vanilla(VanillaPackResources),
    Composite {
        primary: Box<PackResources>,
        stack: Vec<PackResources>,
    },
}

/// ── Common accessors ──

impl PackResources {
    fn common(&self) -> &PackResourcesCommon {
        match self {
            Self::Path(p) => &p.common,
            Self::File(f) => &f.common,
            Self::Vanilla(v) => &v.common,
            Self::Composite { primary, .. } => primary.common(),
        }
    }

    pub fn location(&self) -> &PackLocationInfo {
        &self.common().location
    }

    pub fn pack_id(&self) -> &str {
        &self.location().id
    }

    pub fn known_pack_info(&self) -> Option<&KnownPack> {
        self.location().known_pack_info.as_ref()
    }
}

/// ── Resource methods ──

impl PackResources {
    pub fn get_root_resource(&self, path: &[&str]) -> Option<IoSupplier<Vec<u8>>> {
        match self {
            Self::Path(p) => p.get_root_resource(path),
            Self::File(_) => todo!("FilePackResources.get_root_resource"),
            Self::Vanilla(_) => todo!("VanillaPackResources.get_root_resource"),
            Self::Composite { primary, .. } => primary.get_root_resource(path),
        }
    }

    pub fn get_resource(
        &self,
        pack_type: PackType,
        location: &Identifier,
    ) -> Option<IoSupplier<Vec<u8>>> {
        match self {
            Self::Path(p) => p.get_resource(pack_type, location),
            Self::File(_) => todo!("FilePackResources.get_resource"),
            Self::Vanilla(_) => todo!("VanillaPackResources.get_resource"),
            Self::Composite { primary, stack } => {
                for pack in stack.iter() {
                    if let Some(r) = pack.get_resource(pack_type, location) {
                        return Some(r);
                    }
                }
                primary.get_resource(pack_type, location)
            }
        }
    }

    pub fn list_resources(
        &self,
        pack_type: PackType,
        namespace: &str,
        path: &str,
        output: &mut impl ResourceOutput,
    ) {
        match self {
            Self::Path(p) => p.list_resources(pack_type, namespace, path, output),
            Self::File(_) => todo!("FilePackResources.list_resources"),
            Self::Vanilla(_) => todo!("VanillaPackResources.list_resources"),
            Self::Composite { primary, stack } => {
                let mut seen = std::collections::HashSet::new();
                for pack in stack.iter() {
                    let mut buf = Vec::new();
                    pack.list_resources(pack_type, namespace, path, &mut |id, supplier| {
                        buf.push((id, supplier));
                    });
                    for (id, supplier) in buf.drain(..) {
                        if seen.insert(id.clone()) {
                            output(id, supplier);
                        }
                    }
                }
                let mut buf = Vec::new();
                primary.list_resources(pack_type, namespace, path, &mut |id, supplier| {
                    buf.push((id, supplier));
                });
                for (id, supplier) in buf {
                    if seen.insert(id.clone()) {
                        output(id, supplier);
                    }
                }
            }
        }
    }

    pub fn get_namespaces(&self, pack_type: PackType) -> Vec<String> {
        match self {
            Self::Path(p) => p.get_namespaces(pack_type),
            Self::File(_) => todo!("FilePackResources.get_namespaces"),
            Self::Vanilla(_) => todo!("VanillaPackResources.get_namespaces"),
            Self::Composite { primary, stack } => {
                let mut namespaces: Vec<String> = Vec::new();
                for pack in stack.iter() {
                    for ns in pack.get_namespaces(pack_type) {
                        if !namespaces.contains(&ns) {
                            namespaces.push(ns);
                        }
                    }
                }
                for ns in primary.get_namespaces(pack_type) {
                    if !namespaces.contains(&ns) {
                        namespaces.push(ns);
                    }
                }
                namespaces
            }
        }
    }

    pub fn get_metadata_section<T: DeserializeOwned>(
        &self,
        section: &MetadataSectionType<T>,
    ) -> io::Result<Option<T>> {
        let root_resource = self.get_root_resource(&["pack.mcmeta"]);
        self.common().get_metadata_section(root_resource, section)
    }
}
