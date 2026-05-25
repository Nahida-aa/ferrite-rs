use std::io;
use std::sync::OnceLock;

use serde::de::DeserializeOwned;

use crate::packs::metadata::metadata_section_type::MetadataSectionType;
use crate::packs::pack_location_info::PackLocationInfo;
use crate::packs::resources::io_supplier::IoSupplier;

/// Shared data and logic for all pack types.
///
/// Holds the pack's `PackLocationInfo` and lazily caches the parsed
/// `pack.mcmeta` JSON.
///
/// Java 对照: net.minecraft.server.packs.AbstractPackResources
pub struct PackResourcesCommon {
    pub location: PackLocationInfo,
    metadata: OnceLock<Option<serde_json::Value>>,
}

impl PackResourcesCommon {
    pub fn new(location: PackLocationInfo) -> Self {
        Self {
            location,
            metadata: OnceLock::new(),
        }
    }

    pub fn location(&self) -> &PackLocationInfo {
        &self.location
    }

    /// Java 对照: AbstractPackResources.getMetadataSection
    ///
    /// On the first call reads `pack.mcmeta` via the supplied
    /// `root_resource`, caches the parsed JSON, then returns the
    /// requested section (if present).
    pub fn get_metadata_section<T: DeserializeOwned>(
        &self,
        root_resource: Option<IoSupplier<Vec<u8>>>,
        section: &MetadataSectionType<T>,
    ) -> io::Result<Option<T>> {
        let meta = self.metadata.get_or_init(|| {
            root_resource
                .and_then(|s| s.get().ok())
                .and_then(|data| serde_json::from_slice(&data).ok())
        });
        match meta {
            Some(json) => {
                let section_data = json.get(&section.name);
                match section_data {
                    Some(v) => {
                        let value = serde_json::from_value(v.clone())
                            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                        Ok(Some(value))
                    }
                    None => Ok(None),
                }
            }
            None => Ok(None),
        }
    }
}
