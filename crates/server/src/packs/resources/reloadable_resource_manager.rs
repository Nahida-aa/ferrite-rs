use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use resources::identifier::Identifier;

use crate::packs::pack_resources::PackResources;
use crate::packs::pack_type::PackType;
use crate::packs::resources::multi_pack_resource_manager::MultiPackResourceManager;
use crate::packs::resources::preparable_reload_listener::PreparableReloadListener;
use crate::packs::resources::reload_instance::ReloadInstance;
use crate::packs::resources::resource::Resource;
use crate::packs::resources::resource_manager::{ResourceFilter, ResourceManager};
use crate::packs::resources::resource_provider::ResourceProvider;
use crate::packs::resources::simple_reload_instance;

/// Java 对照: net.minecraft.server.packs.resources.ReloadableResourceManager
pub struct ReloadableResourceManager {
    /// Shared across reload threads; read-only after construction.
    resources: Arc<MultiPackResourceManager>,
    pack_type: PackType,
    listeners: Vec<Arc<dyn PreparableReloadListener<MultiPackResourceManager>>>,
}

impl ReloadableResourceManager {
    pub fn new(pack_type: PackType) -> Self {
        Self {
            resources: Arc::new(MultiPackResourceManager::new(pack_type, Vec::new())),
            pack_type,
            listeners: Vec::new(),
        }
    }

    /// Java 对照: ReloadableResourceManager.registerReloadListener
    pub fn register_reload_listener(
        &mut self,
        listener: Box<dyn PreparableReloadListener<MultiPackResourceManager>>,
    ) {
        self.listeners.push(Arc::from(listener));
    }

    /// Java 对照: ReloadableResourceManager.createReload
    pub fn create_reload(&mut self, packs: Vec<Arc<PackResources>>) -> ReloadInstance {
        let new_manager =
            Arc::new(MultiPackResourceManager::new(self.pack_type, packs));
        let old = std::mem::replace(&mut self.resources, Arc::clone(&new_manager));

        // Close the old manager (no-op for now).
        drop(old);

        let listeners = self.listeners.clone();
        simple_reload_instance::create_reload(new_manager, listeners)
    }
}

impl ResourceProvider for ReloadableResourceManager {
    fn get_resource(&self, location: &Identifier) -> Option<Resource> {
        self.resources.get_resource(location)
    }
}

impl ResourceManager for ReloadableResourceManager {
    fn get_namespaces(&self) -> HashSet<String> {
        self.resources.get_namespaces()
    }

    fn get_resource_stack(&self, location: &Identifier) -> Vec<Resource> {
        self.resources.get_resource_stack(location)
    }

    fn list_resources(
        &self,
        directory: &str,
        filter: &mut impl ResourceFilter,
    ) -> HashMap<Identifier, Resource> {
        self.resources.list_resources(directory, filter)
    }

    fn list_resource_stacks(
        &self,
        directory: &str,
        filter: &mut impl ResourceFilter,
    ) -> HashMap<Identifier, Vec<Resource>> {
        self.resources.list_resource_stacks(directory, filter)
    }

    fn list_packs(&self) -> Vec<&PackResources> {
        self.resources.list_packs()
    }
}

impl Drop for ReloadableResourceManager {
    fn drop(&mut self) {
        // Arc will handle cleanup when all references are dropped.
    }
}
