// Minimal Rust port of Key parts of Java's BlockBehaviour and Properties
// Only implements a small subset required by `BlockState` and consumers.

use std::fmt::Debug;

/// Simplified MapColor placeholder
#[derive(Clone, Copy, Debug)]
pub enum MapColor {
    None,
    // Extend as needed
}

impl Default for MapColor {
    fn default() -> Self {
        MapColor::None
    }
}

/// BlockBehaviour properties: small subset used by BlockState logic.
#[derive(Clone)]
pub struct Properties {
    pub has_collision: bool,
    pub light_emission: u8,
    pub is_air: bool,
    pub destroy_time: f32,
    pub explosion_resistance: f32,
    pub map_color: MapColor,
}

impl Properties {
    pub fn of() -> Self {
        Self {
            has_collision: true,
            light_emission: 0,
            is_air: false,
            destroy_time: 1.0,
            explosion_resistance: 0.0,
            map_color: MapColor::None,
        }
    }

    pub fn air() -> Self {
        Self {
            is_air: true,
            has_collision: false,
            light_emission: 0,
            destroy_time: 0.0,
            explosion_resistance: 0.0,
            map_color: MapColor::None,
        }
    }

    pub fn light_emission(mut self, value: u8) -> Self {
        self.light_emission = value;
        self
    }

    pub fn no_collision(mut self) -> Self {
        self.has_collision = false;
        self
    }
}

/// Minimal BlockBehaviour that holds `Properties` and exposes accessors.
#[derive(Clone)]
pub struct BlockBehaviour {
    pub properties: Properties,
}

impl BlockBehaviour {
    pub fn new(properties: Properties) -> Self {
        Self { properties }
    }

    pub fn properties(&self) -> &Properties {
        &self.properties
    }

    pub fn has_collision(&self) -> bool {
        self.properties.has_collision
    }

    pub fn is_randomly_ticking(&self) -> bool {
        false
    }

    pub fn get_light_emission(&self) -> u8 {
        self.properties.light_emission
    }

    pub fn is_air(&self) -> bool {
        self.properties.is_air
    }

    pub fn default_map_color(&self) -> MapColor {
        self.properties.map_color
    }

    pub fn default_destroy_time(&self) -> f32 {
        self.properties.destroy_time
    }
}

// Minimal BlockStateBase moved here from separate file: caches derived block-state properties.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct BlockStateBase {
    pub light_emission: u8,
    pub is_air: bool,
    pub destroy_speed: f32,
    pub can_occlude: bool,
}

impl BlockStateBase {
    /// Create a default (uninitialized) base.
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize the cache from a `BlockState` and its `BlockBehaviour`.
    pub fn init_from<S>(_state: &S, behaviour: &BlockBehaviour) {
        // NOTE: kept signature generic to avoid circular dependencies when calling.
        // Actual initialization logic is implemented in callers using `behaviour`.
    }

    pub fn get_light_emission(&self) -> u8 {
        self.light_emission
    }

    pub fn is_air(&self) -> bool {
        self.is_air
    }

    pub fn get_destroy_speed(&self) -> f32 {
        self.destroy_speed
    }

    pub fn can_occlude(&self) -> bool {
        self.can_occlude
    }
}
