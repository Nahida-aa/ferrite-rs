/// Minimal `StateHolder` trait to support property-based access used by
/// several systems. This is a pragmatic, string-keyed abstraction that
/// mirrors enough of the Java `StateHolder` for early wiring.
pub trait StateHolder: Sized {
    /// Get a property's string value by property name.
    fn get_value_str(&self, property_name: &str) -> Option<&str>;

    /// Return a new instance with the property set to the given string value.
    fn set_value_str(&self, property_name: &str, value: String) -> Self;

    /// Return an iterator over (property_name, value) pairs.
    fn values<'a>(&'a self) -> Box<dyn Iterator<Item = (&'a str, &'a str)> + 'a>;
}

/// Concrete data container that holds owner and property arrays.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct StateHolderData {
    pub owner: String,
    pub property_keys: Vec<String>,
    pub property_values: Vec<String>,
}

impl StateHolderData {
    pub fn new(
        owner: impl Into<String>,
        property_keys: Vec<String>,
        property_values: Vec<String>,
    ) -> Self {
        Self {
            owner: owner.into(),
            property_keys,
            property_values,
        }
    }

    pub fn default() -> Self {
        Self {
            owner: "minecraft:air".to_string(),
            property_keys: Vec::new(),
            property_values: Vec::new(),
        }
    }
}

impl StateHolder for StateHolderData {
    fn get_value_str(&self, property_name: &str) -> Option<&str> {
        self.property_keys
            .iter()
            .position(|k| k == property_name)
            .map(|i| self.property_values[i].as_str())
    }

    fn set_value_str(&self, property_name: &str, value: String) -> Self {
        let mut keys = self.property_keys.clone();
        let mut values = self.property_values.clone();
        if let Some(pos) = keys.iter().position(|k| k == property_name) {
            values[pos] = value;
        } else {
            keys.push(property_name.to_string());
            values.push(value);
        }
        Self {
            owner: self.owner.clone(),
            property_keys: keys,
            property_values: values,
        }
    }

    fn values<'a>(&'a self) -> Box<dyn Iterator<Item = (&'a str, &'a str)> + 'a> {
        Box::new(self.property_keys.iter().map(move |k| {
            let v = self.get_value_str(k).unwrap_or("");
            (k.as_str(), v)
        }))
    }
}
