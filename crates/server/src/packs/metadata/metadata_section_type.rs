use std::marker::PhantomData;

/// Java 对照: net.minecraft.server.packs.metadata.MetadataSectionType
/// Rust 直接用 serde::Serialize + serde::Deserialize 代替 DFU Codec
pub struct MetadataSectionType<T> {
    pub name: String,
    pub _phantom: PhantomData<T>,
}

impl<T> MetadataSectionType<T> {
    pub fn new(name: String) -> Self {
        Self { name, _phantom: PhantomData }
    }

    pub fn with_value(&self, value: T) -> WithValue<'_, T> {
        WithValue { type_: self, value }
    }
}

pub struct WithValue<'a, T> {
    pub type_: &'a MetadataSectionType<T>,
    pub value: T,
}

impl<'a, T> WithValue<'a, T> {
    pub fn unwrap_to_type<U>(&self, type_: &MetadataSectionType<U>) -> Option<&U> {
        if std::ptr::from_ref(self.type_) as *const () == std::ptr::from_ref(type_) as *const () {
            Some(unsafe { &*(&self.value as *const T as *const U) })
        } else {
            None
        }
    }
}
