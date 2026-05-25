use serialization::codec::Codec;

pub struct MetadataSectionType<T: 'static + Send + Sync> {
    pub name: String,
    pub codec: Codec<T>,
}

impl<T: 'static + Send + Sync> MetadataSectionType<T> {
    pub fn new(name: String, codec: Codec<T>) -> Self {
        Self { name, codec }
    }

    pub fn with_value(&self, value: T) -> WithValue<'_, T> {
        WithValue { type_: self, value }
    }
}

pub struct WithValue<'a, T: 'static + Send + Sync> {
    pub type_: &'a MetadataSectionType<T>,
    pub value: T,
}
