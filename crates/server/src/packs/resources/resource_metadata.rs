use std::any::Any;
use std::collections::HashMap;
use std::io::Read;

use serde::de::DeserializeOwned;

use crate::packs::metadata::metadata_section_type::{MetadataSectionType, WithValue};

pub trait ResourceMetadata {
    fn get_section<T: DeserializeOwned + Clone + 'static>(
        &self,
        serializer: &MetadataSectionType<T>,
    ) -> Option<T>;

    fn get_typed_section<'a, T: DeserializeOwned + Clone + 'static>(
        &self,
        type_: &'a MetadataSectionType<T>,
    ) -> Option<WithValue<'a, T>> {
        self.get_section(type_).map(move |v| type_.with_value(v))
    }
}

#[derive(Clone, Copy)]
pub struct EmptyMetadata;

impl ResourceMetadata for EmptyMetadata {
    fn get_section<T: DeserializeOwned + Clone + 'static>(
        &self,
        _serializer: &MetadataSectionType<T>,
    ) -> Option<T> {
        None
    }
}

pub const EMPTY: EmptyMetadata = EmptyMetadata;

use crate::packs::resources::io_supplier::IoSupplier;

pub struct EmptyMetadataSupplier;
impl IoSupplier for EmptyMetadataSupplier {
    type Output = EmptyMetadata;
    fn get(&self) -> std::io::Result<EmptyMetadata> {
        Ok(EMPTY)
    }
}
pub const EMPTY_SUPPLIER: EmptyMetadataSupplier = EmptyMetadataSupplier;

pub struct JsonBackedMetadata {
    json: serde_json::Value,
}

impl JsonBackedMetadata {
    pub fn from_json_stream<R: Read>(reader: R) -> Result<Self, serde_json::Error> {
        let json: serde_json::Value = serde_json::from_reader(reader)?;
        Ok(Self { json })
    }
}

impl ResourceMetadata for JsonBackedMetadata {
    fn get_section<T: DeserializeOwned + Clone + 'static>(
        &self,
        serializer: &MetadataSectionType<T>,
    ) -> Option<T> {
        let section = self.json.get(&serializer.name)?;
        serde_json::from_value(section.clone()).ok()
    }
}

pub struct MapBased {
    values: HashMap<String, Box<dyn Any>>,
}

impl MapBased {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn insert<T: Clone + 'static>(&mut self, type_: &MetadataSectionType<T>, value: T) {
        self.values.insert(type_.name.clone(), Box::new(value));
    }
}

impl ResourceMetadata for MapBased {
    fn get_section<T: DeserializeOwned + Clone + 'static>(
        &self,
        serializer: &MetadataSectionType<T>,
    ) -> Option<T> {
        self.values
            .get(&serializer.name)?
            .downcast_ref::<T>()
            .cloned()
    }
}

pub fn of<T: DeserializeOwned + Clone + 'static>(
    type_: &MetadataSectionType<T>,
    value: T,
) -> MapBased {
    let mut map = MapBased::new();
    map.insert(type_, value);
    map
}
