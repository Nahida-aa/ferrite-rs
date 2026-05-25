use crate::packs::resources::resource_manager::ResourceManager;

pub trait CloseableResourceManager: ResourceManager {
    fn close(&mut self);
}
