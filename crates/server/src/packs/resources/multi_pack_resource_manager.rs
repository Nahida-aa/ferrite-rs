use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use resources::identifier::Identifier;

use crate::packs::pack_resources::PackResources;
use crate::packs::pack_type::PackType;
use crate::packs::resources::closeable_resource_manager::CloseableResourceManager;
use crate::packs::resources::fallback_resource_manager::FallbackResourceManager;
use crate::packs::resources::resource::Resource;
use crate::packs::resources::resource_manager::{ResourceFilter, ResourceManager};
use crate::packs::resources::resource_provider::ResourceProvider;

pub struct MultiPackResourceManager {
    namespaced_managers: HashMap<String, FallbackResourceManager>,
    packs: Vec<Arc<PackResources>>,
}

impl MultiPackResourceManager {
    pub fn new(pack_type: PackType, packs: Vec<Arc<PackResources>>) -> Self {
        let mut namespaced_managers: HashMap<String, FallbackResourceManager> = HashMap::new();

        // Collect all namespaces from all packs
        let all_namespaces: Vec<String> = packs
            .iter()
            .flat_map(|p| p.get_namespaces(pack_type))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        for pack in &packs {
            let provided_namespaces: HashSet<String> =
                pack.get_namespaces(pack_type).into_iter().collect();

            for ns in &all_namespaces {
                let pack_has_ns = provided_namespaces.contains(ns);

                let mgr = namespaced_managers
                    .entry(ns.clone())
                    .or_insert_with(|| FallbackResourceManager::new(pack_type, ns.clone()));

                if pack_has_ns {
                    mgr.push(Arc::clone(pack));
                }
            }
        }

        Self {
            namespaced_managers,
            packs,
        }
    }
}

impl ResourceProvider for MultiPackResourceManager {
    fn get_resource(&self, location: &Identifier) -> Option<Resource> {
        self.namespaced_managers
            .get(location.namespace())
            .and_then(|mgr| mgr.get_resource(location))
    }
}

impl ResourceManager for MultiPackResourceManager {
    fn get_namespaces(&self) -> HashSet<String> {
        self.namespaced_managers.keys().cloned().collect()
    }

    fn get_resource_stack(&self, location: &Identifier) -> Vec<Resource> {
        self.namespaced_managers
            .get(location.namespace())
            .map(|mgr| mgr.get_resource_stack(location))
            .unwrap_or_default()
    }

    fn list_resources(
        &self,
        directory: &str,
        filter: &mut impl ResourceFilter,
    ) -> HashMap<Identifier, Resource> {
        Self::check_trailing_directory_path(directory);
        let mut result = HashMap::new();
        for mgr in self.namespaced_managers.values() {
            result.extend(mgr.list_resources(directory, filter));
        }
        result
    }

    fn list_resource_stacks(
        &self,
        directory: &str,
        filter: &mut impl ResourceFilter,
    ) -> HashMap<Identifier, Vec<Resource>> {
        Self::check_trailing_directory_path(directory);
        let mut result = HashMap::new();
        for mgr in self.namespaced_managers.values() {
            result.extend(mgr.list_resource_stacks(directory, filter));
        }
        result
    }

    fn list_packs(&self) -> Vec<&PackResources> {
        self.packs.iter().map(|arc| arc.as_ref()).collect()
    }
}

impl CloseableResourceManager for MultiPackResourceManager {
    fn close(&mut self) {
        // PackResources don't have a close yet — no-op for now
    }
}

impl MultiPackResourceManager {
    fn check_trailing_directory_path(directory: &str) {
        if directory.ends_with('/') {
            panic!("Trailing slash in path {}", directory);
        }
    }
}
