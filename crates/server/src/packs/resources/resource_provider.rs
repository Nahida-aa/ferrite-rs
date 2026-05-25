use std::io;
use std::sync::Arc;

use resources::identifier::Identifier;

use crate::packs::resources::resource::Resource;

pub trait ResourceProvider: Send + Sync {
    fn get_resource(&self, location: &Identifier) -> Option<Resource>;

    fn get_resource_or_throw(&self, location: &Identifier) -> io::Result<Resource> {
        self.get_resource(location)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, location.to_string()))
    }

    fn open(&self, location: &Identifier) -> io::Result<Vec<u8>> {
        self.get_resource_or_throw(location)?.open()
    }
}

impl<T: ResourceProvider + ?Sized> ResourceProvider for Box<T> {
    fn get_resource(&self, location: &Identifier) -> Option<Resource> {
        (**self).get_resource(location)
    }
}

impl<T: ResourceProvider + ?Sized> ResourceProvider for Arc<T> {
    fn get_resource(&self, location: &Identifier) -> Option<Resource> {
        (**self).get_resource(location)
    }
}
