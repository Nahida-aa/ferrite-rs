use std::io;

use crate::packs::repository::known_pack::KnownPack;
use crate::packs::resources::io_supplier::IoSupplier;
use crate::packs::resources::resource_metadata::ResourceMetadataEnum;

/// A resource (a file obtained from a pack), bundled with its metadata.
///
pub struct Resource {
    source_pack_id: String,
    known_pack: Option<KnownPack>,
    stream_supplier: IoSupplier<Vec<u8>>,
    metadata: ResourceMetadataEnum,
}

impl Resource {
    pub fn new(
        source_pack_id: String,
        known_pack: Option<KnownPack>,
        stream_supplier: IoSupplier<Vec<u8>>,
        metadata_supplier: impl FnOnce() -> io::Result<ResourceMetadataEnum>,
    ) -> io::Result<Self> {
        let metadata = metadata_supplier()?;
        Ok(Self {
            source_pack_id,
            known_pack,
            stream_supplier,
            metadata,
        })
    }

    /// Convenience constructor — no metadata supplier, defaults to `Empty`.
    pub fn with_source_and_stream(
        source_pack_id: String,
        known_pack: Option<KnownPack>,
        stream_supplier: IoSupplier<Vec<u8>>,
    ) -> Self {
        Self {
            source_pack_id,
            known_pack,
            stream_supplier,
            metadata: ResourceMetadataEnum::Empty,
        }
    }

    /// Java 对照: Resource.sourcePackId()
    pub fn source_pack_id(&self) -> &str {
        &self.source_pack_id
    }

    /// Java 对照: Resource.knownPackInfo()
    pub fn known_pack_info(&self) -> Option<&KnownPack> {
        self.known_pack.as_ref()
    }

    /// Java 对照: Resource.open()
    pub fn open(&self) -> io::Result<Vec<u8>> {
        self.stream_supplier.get()
    }

    /// Java 对照: Resource.metadata()
    pub fn metadata(&self) -> &ResourceMetadataEnum {
        &self.metadata
    }
}
