/// Simplified Material representation for model sprites.
///
/// This mirrors the Java `Material` record in a minimal way: it stores a
/// sprite identifier (as a string `sprite`) and a `force_translucent` flag.
/// The `Baked` variant contains a resolved sprite reference (as string for now).

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Material {
    pub sprite: String,
    pub force_translucent: bool,
}

impl Material {
    pub fn new<S: Into<String>>(sprite: S) -> Self {
        Self {
            sprite: sprite.into(),
            force_translucent: false,
        }
    }

    pub fn with_force_translucent(mut self, force: bool) -> Self {
        self.force_translucent = force;
        self
    }
}

/// Baked material produced by a `ModelBaker`/`MaterialBaker`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Baked {
    /// Resolved sprite id (atlas lookup happens elsewhere).
    pub sprite: String,
    pub force_translucent: bool,
}

impl Baked {
    pub fn new<S: Into<String>>(sprite: S, force_translucent: bool) -> Self {
        Self {
            sprite: sprite.into(),
            force_translucent,
        }
    }
}

impl From<Material> for Baked {
    fn from(m: Material) -> Self {
        // Default baked conversion leaves sprite id as-is; actual baker should
        // replace with resolved atlas sprite when available.
        Baked::new(m.sprite, m.force_translucent)
    }
}
