use std::fmt;
use std::sync::Arc;

/// Minimal generic `Property<T>` representation.
///
/// This is a lightweight, pragmatic port of the Java class used by
/// `BlockState` and state holders. It stores a list of allowed values and
/// corresponding string names. The implementation is intentionally simple
/// (linear lookups) — optimizations can be added later.
#[derive(Clone)]
pub struct Property<T: Clone + PartialEq + fmt::Debug> {
    name: String,
    possible_values: Vec<T>,
    value_names: Vec<String>,
}

impl<T: Clone + PartialEq + fmt::Debug> Property<T> {
    /// Construct a property from a name and a sequence of `(value, name)` pairs.
    pub fn new(name: impl Into<String>, values: Vec<(T, impl Into<String>)>) -> Self {
        let name = name.into();
        let mut possible_values = Vec::with_capacity(values.len());
        let mut value_names = Vec::with_capacity(values.len());
        for (v, n) in values {
            possible_values.push(v);
            value_names.push(n.into());
        }
        Self {
            name,
            possible_values,
            value_names,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get_possible_values(&self) -> &[T] {
        &self.possible_values
    }

    /// Return the textual name for a specific value, or `None` if not found.
    pub fn get_name(&self, value: &T) -> Option<&str> {
        self.possible_values
            .iter()
            .position(|v| v == value)
            .map(|i| self.value_names[i].as_str())
    }

    /// Parse a textual value name into the corresponding value.
    pub fn get_value(&self, name: &str) -> Option<T> {
        self.value_names
            .iter()
            .position(|n| n == name)
            .map(|i| self.possible_values[i].clone())
    }

    /// Internal index used for compact storage/ordering.
    pub fn get_internal_index(&self, value: &T) -> Option<usize> {
        self.possible_values.iter().position(|v| v == value)
    }
}

/// A `Value` couples a `Property` with one of its allowed values.
#[derive(Clone)]
pub struct Value<T: Clone + PartialEq + fmt::Debug> {
    pub property: Arc<Property<T>>,
    pub value: T,
}

impl<T: Clone + PartialEq + fmt::Debug> Value<T> {
    pub fn new(property: Arc<Property<T>>, value: T) -> Self {
        // Validate membership
        if property.get_internal_index(&value).is_none() {
            panic!(
                "Value {:?} does not belong to property {}",
                value,
                property.name()
            );
        }
        Self { property, value }
    }

    pub fn value_name(&self) -> String {
        self.property
            .get_name(&self.value)
            .map(|s| s.to_string())
            .unwrap_or_default()
    }
}

impl<T: Clone + PartialEq + fmt::Debug> fmt::Display for Value<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}={}", self.property.name(), self.value_name())
    }
}

impl<T: Clone + PartialEq + fmt::Debug> fmt::Debug for Property<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Property")
            .field("name", &self.name)
            .field("values_count", &self.possible_values.len())
            .finish()
    }
}
