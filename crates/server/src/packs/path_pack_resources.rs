use std::path::{Path, PathBuf};

use resources::identifier::Identifier;

use crate::packs::abstract_pack_resources::PackResourcesCommon;
use crate::packs::pack_location_info::PackLocationInfo;
use crate::packs::pack_type::PackType;
use crate::packs::pack_resources::ResourceOutput;
use crate::packs::resources::io_supplier::IoSupplier;

/// A pack backed by a directory on the filesystem.
///
/// Java 对照: net.minecraft.server.packs.PathPackResources
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

    pub fn top_pack_dir(&self, pack_type: PackType) -> PathBuf {
        self.root.join(pack_type.directory())
    }

    pub fn get_resource_inner(top_dir: &Path, location: &Identifier) -> Option<IoSupplier<Vec<u8>>> {
        let path = top_dir
            .join(location.namespace())
            .join(location.path());
        if path.exists() {
            Some(IoSupplier::File(path))
        } else {
            None
        }
    }

    pub fn get_root_resource(&self, path: &[&str]) -> Option<IoSupplier<Vec<u8>>> {
        let full_path = self.root.join(path.join("/"));
        if full_path.exists() {
            Some(IoSupplier::File(full_path))
        } else {
            None
        }
    }

    pub fn get_resource(&self, pack_type: PackType, location: &Identifier) -> Option<IoSupplier<Vec<u8>>> {
        let top_dir = self.top_pack_dir(pack_type);
        Self::get_resource_inner(&top_dir, location)
    }

    fn list_path_namespace(
        dir: &Path,
        namespace: &str,
        prefix: &Path,
        output: &mut impl ResourceOutput,
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

    pub fn list_resources(
        &self,
        pack_type: PackType,
        namespace: &str,
        path: &str,
        output: &mut impl ResourceOutput,
    ) {
        let top_dir = self.top_pack_dir(pack_type);
        let target_dir = top_dir.join(namespace);
        let full_prefix = target_dir.clone();
        let search_dir = target_dir.join(path);
        if search_dir.is_dir() {
            Self::list_path_namespace(&search_dir, namespace, &full_prefix, output);
        }
    }

    pub fn get_namespaces(&self, pack_type: PackType) -> Vec<String> {
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
