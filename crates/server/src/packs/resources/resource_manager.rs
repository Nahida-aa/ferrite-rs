use std::collections::{HashMap, HashSet};

use resources::identifier::Identifier;

use crate::packs::pack_resources::PackResources;
use crate::packs::resources::resource::Resource;
use crate::packs::resources::resource_provider::ResourceProvider;

/// Java 对照: java.util.function.Predicate<Identifier>
pub trait ResourceFilter: FnMut(&Identifier) -> bool {}
impl<T: FnMut(&Identifier) -> bool> ResourceFilter for T {}

pub trait ResourceManager: ResourceProvider {
    fn get_namespaces(&self) -> HashSet<String>;

    fn get_resource_stack(&self, location: &Identifier) -> Vec<Resource>;

    fn list_resources(
        &self,
        directory: &str,
        filter: &mut impl ResourceFilter,
    ) -> HashMap<Identifier, Resource>;

    fn list_resource_stacks(
        &self,
        directory: &str,
        filter: &mut impl ResourceFilter,
    ) -> HashMap<Identifier, Vec<Resource>>;

    fn list_packs(&self) -> Vec<&PackResources>;
}

/// Java 对照: ResourceManager.Empty
pub struct EmptyResourceManager;

impl ResourceProvider for EmptyResourceManager {
    fn get_resource(&self, _location: &Identifier) -> Option<Resource> {
        None
    }
}

impl ResourceManager for EmptyResourceManager {
    fn get_namespaces(&self) -> HashSet<String> {
        HashSet::new()
    }

    fn get_resource_stack(&self, _location: &Identifier) -> Vec<Resource> {
        Vec::new()
    }

    fn list_resources(
        &self,
        _directory: &str,
        _filter: &mut impl ResourceFilter,
    ) -> HashMap<Identifier, Resource> {
        HashMap::new()
    }

    fn list_resource_stacks(
        &self,
        _directory: &str,
        _filter: &mut impl ResourceFilter,
    ) -> HashMap<Identifier, Vec<Resource>> {
        HashMap::new()
    }

    fn list_packs(&self) -> Vec<&PackResources> {
        Vec::new()
    }
}
