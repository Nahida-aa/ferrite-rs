use std::io;
use std::path::{Path, PathBuf};

use resources::identifier::Identifier;
use serde::de::DeserializeOwned;

use crate::packs::abstract_pack_resources::PackResourcesCommon;
use crate::packs::metadata::metadata_section_type::MetadataSectionType;
use crate::packs::pack_location_info::PackLocationInfo;
use crate::packs::pack_type::PackType;
use crate::packs::repository::known_pack::KnownPack;
use crate::packs::resources::io_supplier::IoSupplier;

/// Java 对照: net.minecraft.server.packs.PackResources.ResourceOutput
pub type ResourceOutput<'a> = dyn FnMut(Identifier, IoSupplier<Vec<u8>>) + 'a;

/// ── PathPackResources (Java 对照: PathPackResources) ──

pub struct PathPackResources {
    pub common: PackResourcesCommon,
    pub root: PathBuf,
}

impl PathPackResources {
    pub fn new(location: PackLocationInfo, root: PathBuf) -> Self {
        Self {
            common: PackResourcesCommon::new(location),
            root,
        }
    }

    fn top_pack_dir(&self, pack_type: PackType) -> PathBuf {
        self.root.join(pack_type.directory())
    }

    fn get_resource_inner(top_dir: &Path, location: &Identifier) -> Option<IoSupplier<Vec<u8>>> {
        let path = top_dir
            .join(location.namespace())
            .join(location.path());
        if path.exists() {
            Some(IoSupplier::File(path))
        } else {
            None
        }
    }

    fn list_path_namespace(
        dir: &Path,
        namespace: &str,
        prefix: &Path,
        output: &mut ResourceOutput<'_>,
    ) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    Self::list_path_namespace(&path, namespace, prefix, output);
                } else if path.is_file() {
                    if let Ok(relative) = path.strip_prefix(prefix) {
                        let resource_path = relative.to_string_lossy().replace('\\', "/");
                        if let Some(id) = Identifier::try_build(namespace, &resource_path) {
                            output(id, IoSupplier::File(path));
                        }
                    }
                }
            }
        }
    }

    fn do_list_resources(
        &self,
        pack_type: PackType,
        namespace: &str,
        path: &str,
        output: &mut ResourceOutput<'_>,
    ) {
        let top_dir = self.top_pack_dir(pack_type);
        let target_dir = top_dir.join(namespace);
        let full_prefix = target_dir.clone();
        let search_dir = target_dir.join(path);
        if search_dir.is_dir() {
            Self::list_path_namespace(&search_dir, namespace, &full_prefix, output);
        }
    }

    fn do_get_namespaces(&self, pack_type: PackType) -> Vec<String> {
        let top_dir = self.top_pack_dir(pack_type);
        let mut namespaces = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&top_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if Identifier::is_valid_namespace(&name) {
                        namespaces.push(name);
                    }
                }
            }
        }
        namespaces
    }
}

/// ── FilePackResources (Java 对照: FilePackResources) ──

pub struct FilePackResources {
    pub common: PackResourcesCommon,
    // TODO: zip file access
}

/// ── VanillaPackResources (Java 对照: VanillaPackResources) ──

pub struct VanillaPackResources {
    pub common: PackResourcesCommon,
    // TODO: namespace list, root paths, paths for each type
}

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
            Self::Path(p) => {
                let full_path = p.root.join(path.join("/"));
                if full_path.exists() {
                    Some(IoSupplier::File(full_path))
                } else {
                    None
                }
            }
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
            Self::Path(p) => PathPackResources::get_resource_inner(&p.top_pack_dir(pack_type), location),
            Self::File(_) => todo!("FilePackResources.get_resource"),
            Self::Vanilla(_) => todo!("VanillaPackResources.get_resource"),
            Self::Composite { stack, .. } => {
                for pack in stack.iter().rev() {
                    if let Some(r) = pack.get_resource(pack_type, location) {
                        return Some(r);
                    }
                }
                None
            }
        }
    }

    pub fn list_resources(
        &self,
        pack_type: PackType,
        namespace: &str,
        path: &str,
        output: &mut ResourceOutput<'_>,
    ) {
        match self {
            Self::Path(p) => p.do_list_resources(pack_type, namespace, path, output),
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
            Self::Path(p) => p.do_get_namespaces(pack_type),
            Self::File(_) => todo!("FilePackResources.get_namespaces"),
            Self::Vanilla(_) => todo!("VanillaPackResources.get_namespaces"),
            Self::Composite { primary, stack } => {
                let mut namespaces: Vec<String> = Vec::new();
                for pack in stack.iter().rev() {
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
