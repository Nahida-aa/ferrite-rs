use std::collections::{HashMap, HashSet};
use std::io;
use std::sync::Arc;

use resources::identifier::Identifier;

use crate::packs::pack_resources::PackResources;
use crate::packs::pack_type::PackType;
use crate::packs::resources::io_supplier::IoSupplier;
use crate::packs::resources::resource::Resource;
use crate::packs::resources::resource_manager::{ResourceFilter, ResourceManager};
use crate::packs::resources::resource_metadata::{JsonBackedMetadata, ResourceMetadataEnum};
use crate::packs::resources::resource_provider::ResourceProvider;

pub struct FallbackResourceManager {
    fallbacks: Vec<PackEntry>,
    pack_type: PackType,
    namespace: String,
}

struct PackEntry {
    name: String,
    resources: Option<Arc<PackResources>>,
    filter: Option<Box<dyn Fn(&Identifier) -> bool + Send + Sync>>,
}

impl PackEntry {
    fn is_filtered(&self, location: &Identifier) -> bool {
        self.filter.as_ref().is_some_and(|f| f(location))
    }
}

fn is_metadata(location: &Identifier) -> bool {
    location.path().ends_with(".mcmeta")
}

fn metadata_location(location: &Identifier) -> Identifier {
    let path = format!("{}.mcmeta", location.path());
    Identifier::try_build(location.namespace(), &path)
        .expect("appending .mcmeta should remain valid")
}

fn resource_from_metadata(location: &Identifier) -> Identifier {
    let path = &location.path()[..location.path().len() - ".mcmeta".len()];
    Identifier::try_build(location.namespace(), path)
        .expect("stripping .mcmeta should remain valid")
}

fn read_metadata(supplier: IoSupplier<Vec<u8>>) -> ResourceMetadataEnum {
    let data = match supplier.get() {
        Ok(d) => d,
        Err(_) => return ResourceMetadataEnum::Empty,
    };
    let mut cursor = io::Cursor::new(data);
    match JsonBackedMetadata::from_json_stream(&mut cursor) {
        Ok(meta) => ResourceMetadataEnum::Json(meta),
        Err(e) => {
            tracing::warn!("Failed to parse resource metadata: {e}");
            ResourceMetadataEnum::Empty
        }
    }
}

impl FallbackResourceManager {
    pub fn new(pack_type: PackType, namespace: impl Into<String>) -> Self {
        Self {
            fallbacks: Vec::new(),
            pack_type,
            namespace: namespace.into(),
        }
    }

    pub fn push(&mut self, pack: Arc<PackResources>) {
        self.push_internal(pack.pack_id().to_string(), Some(pack), None);
    }

    pub fn push_filtered(
        &mut self,
        pack: Arc<PackResources>,
        filter: impl Fn(&Identifier) -> bool + Send + Sync + 'static,
    ) {
        self.push_internal(
            pack.pack_id().to_string(),
            Some(pack),
            Some(Box::new(filter)),
        );
    }

    pub fn push_filter_only(
        &mut self,
        name: impl Into<String>,
        filter: impl Fn(&Identifier) -> bool + Send + Sync + 'static,
    ) {
        self.push_internal(name.into(), None, Some(Box::new(filter)));
    }

    fn push_internal(
        &mut self,
        name: String,
        resources: Option<Arc<PackResources>>,
        filter: Option<Box<dyn Fn(&Identifier) -> bool + Send + Sync>>,
    ) {
        self.fallbacks.push(PackEntry {
            name,
            resources,
            filter,
        });
    }

    fn create_resource(
        &self,
        pack: &PackResources,
        stream: IoSupplier<Vec<u8>>,
        meta_supplier: Option<IoSupplier<Vec<u8>>>,
    ) -> Resource {
        let metadata = meta_supplier.map_or(ResourceMetadataEnum::Empty, read_metadata);
        let known_pack_info = pack.known_pack_info().cloned();
        Resource::with_metadata(
            pack.pack_id().to_string(),
            known_pack_info,
            stream,
            metadata,
        )
    }

    /// Searches from highest-priority pack down to `final_index`
    /// for a metadata file at `location`.mcmeta.
    fn find_metadata(
        &self,
        location: &Identifier,
        final_index: usize,
    ) -> Option<IoSupplier<Vec<u8>>> {
        let meta_loc = metadata_location(location);
        for i in (final_index..self.fallbacks.len()).rev() {
            let entry = &self.fallbacks[i];
            if entry.is_filtered(&meta_loc) {
                break;
            }
            if let Some(ref pack) = entry.resources {
                if let Some(stream) = pack.get_resource(self.pack_type, &meta_loc) {
                    return Some(stream);
                }
            }
        }
        None
    }
}

impl ResourceProvider for FallbackResourceManager {
    fn get_resource(&self, location: &Identifier) -> Option<Resource> {
        for i in (0..self.fallbacks.len()).rev() {
            let entry = &self.fallbacks[i];
            if let Some(ref pack) = entry.resources {
                if let Some(stream) = pack.get_resource(self.pack_type, location) {
                    let meta = self.find_metadata(location, i);
                    return Some(self.create_resource(pack, stream, meta));
                }
            }
            if entry.is_filtered(location) {
                tracing::warn!(
                    "Resource {} not found, but was filtered by pack {}",
                    location,
                    entry.name
                );
                return None;
            }
        }
        None
    }
}

impl ResourceManager for FallbackResourceManager {
    fn get_namespaces(&self) -> HashSet<String> {
        HashSet::from([self.namespace.clone()])
    }

    fn get_resource_stack(&self, location: &Identifier) -> Vec<Resource> {
        let meta_loc = metadata_location(location);
        let mut result: Vec<Resource> = Vec::new();
        let mut filter_meta = false;
        let mut last_filter_name: Option<&str> = None;

        for i in (0..self.fallbacks.len()).rev() {
            let entry = &self.fallbacks[i];
            if let Some(ref pack) = entry.resources {
                if let Some(stream) = pack.get_resource(self.pack_type, location) {
                    let meta = if filter_meta {
                        None
                    } else {
                        pack.get_resource(self.pack_type, &meta_loc)
                    };
                    let known_pack_info = pack.known_pack_info().cloned();
                    let metadata = meta.map_or(ResourceMetadataEnum::Empty, read_metadata);
                    result.push(Resource::with_metadata(
                        pack.pack_id().to_string(),
                        known_pack_info,
                        stream,
                        metadata,
                    ));
                }
            }
            if entry.is_filtered(location) {
                last_filter_name = Some(entry.name.as_str());
                break;
            }
            if entry.is_filtered(&meta_loc) {
                filter_meta = true;
            }
        }

        if result.is_empty() {
            if let Some(name) = last_filter_name {
                tracing::warn!(
                    "Resource {} not found, but was filtered by pack {}",
                    location,
                    name
                );
            }
        }

        result.reverse();
        result
    }

    fn list_resources(
        &self,
        directory: &str,
        filter: &mut impl ResourceFilter,
    ) -> HashMap<Identifier, Resource> {
        struct TopEntry {
            pack: Arc<PackResources>,
            stream: IoSupplier<Vec<u8>>,
            pack_index: usize,
        }

        let mut top_file: HashMap<Identifier, TopEntry> = HashMap::new();
        let mut top_meta: HashMap<Identifier, TopEntry> = HashMap::new();

        for (i, entry) in self.fallbacks.iter().enumerate() {
            // Remove resources that this entry's filter blocks
            if let Some(ref filter) = entry.filter {
                top_file.retain(|k, _| !filter(k));
                top_meta.retain(|k, _| !filter(k));
            }

            let Some(ref pack) = entry.resources else {
                continue;
            };

            let pack_idx = i;
            let pack_clone = Arc::clone(pack);
            pack.list_resources(
                self.pack_type,
                &self.namespace,
                directory,
                &mut |id, stream| {
                    if is_metadata(&id) {
                        let actual_id = resource_from_metadata(&id);
                        if filter(&actual_id) {
                            top_meta.insert(
                                id,
                                TopEntry {
                                    pack: Arc::clone(&pack_clone),
                                    stream,
                                    pack_index: pack_idx,
                                },
                            );
                        }
                    } else if filter(&id) {
                        top_file.insert(
                            id,
                            TopEntry {
                                pack: Arc::clone(&pack_clone),
                                stream,
                                pack_index: pack_idx,
                            },
                        );
                    }
                },
            );
        }

        let mut result: HashMap<Identifier, Resource> = HashMap::new();
        for (location, te) in top_file {
            let meta_loc = metadata_location(&location);
            let meta = top_meta.get(&meta_loc).and_then(|m| {
                if m.pack_index >= te.pack_index {
                    Some(m.stream.clone())
                } else {
                    None
                }
            });
            result.insert(
                location,
                Resource::with_metadata(
                    te.pack.pack_id().to_string(),
                    te.pack.known_pack_info().cloned(),
                    te.stream,
                    meta.map_or(ResourceMetadataEnum::Empty, read_metadata),
                ),
            );
        }

        result
    }

    fn list_resource_stacks(
        &self,
        directory: &str,
        filter: &mut impl ResourceFilter,
    ) -> HashMap<Identifier, Vec<Resource>> {
        struct StackEntry {
            file_location: Identifier,
            file_sources: Vec<(Arc<PackResources>, IoSupplier<Vec<u8>>)>,
            meta_sources: Vec<(Arc<PackResources>, IoSupplier<Vec<u8>>)>,
        }

        let mut found: HashMap<Identifier, StackEntry> = HashMap::new();

        for entry in &self.fallbacks {
            let Some(ref pack) = entry.resources else {
                continue;
            };

            let pack_clone = Arc::clone(pack);
            pack.list_resources(
                self.pack_type,
                &self.namespace,
                directory,
                &mut |id, stream| {
                    if is_metadata(&id) {
                        let actual_id = resource_from_metadata(&id);
                        if !filter(&actual_id) {
                            return;
                        }
                        found
                            .entry(actual_id)
                            .or_insert_with(|| StackEntry {
                                file_location: Identifier::try_build("", "").expect("placeholder"),
                                file_sources: Vec::new(),
                                meta_sources: Vec::new(),
                            })
                            .meta_sources
                            .push((Arc::clone(&pack_clone), stream));
                    } else {
                        if !filter(&id) {
                            return;
                        }
                        found
                            .entry(id.clone())
                            .or_insert_with(|| StackEntry {
                                file_location: id.clone(),
                                file_sources: Vec::new(),
                                meta_sources: Vec::new(),
                            })
                            .file_sources
                            .push((Arc::clone(&pack_clone), stream));
                    }
                },
            );
        }

        let mut result: HashMap<Identifier, Vec<Resource>> = HashMap::new();
        for (_location, stack) in found {
            if stack.file_sources.is_empty() {
                continue;
            }
            let resources: Vec<Resource> = stack
                .file_sources
                .into_iter()
                .map(|(pack, stream)| {
                    let meta = stack
                        .meta_sources
                        .iter()
                        .find(|(m_pack, _)| Arc::ptr_eq(m_pack, &pack))
                        .map(|(_, ms)| ms.clone());
                    let known_pack_info = pack.known_pack_info().cloned();
                    let metadata = meta.map_or(ResourceMetadataEnum::Empty, read_metadata);
                    Resource::with_metadata(
                        pack.pack_id().to_string(),
                        known_pack_info,
                        stream,
                        metadata,
                    )
                })
                .collect();
            result.insert(stack.file_location, resources);
        }

        result
    }

    fn list_packs(&self) -> Vec<&PackResources> {
        self.fallbacks
            .iter()
            .filter_map(|e| e.resources.as_ref().map(|arc| arc.as_ref()))
            .collect()
    }
}
