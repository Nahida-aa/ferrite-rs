// Simplified, portable Rust port of the Java
// net.minecraft.world.level.block.state.BlockState
// This is a minimal, self-contained representation used as a starting point
// for wiring codecs and registry lookups. It intentionally avoids depending
// on the rest of the (not-yet-ported) Java-style block system.

use crate::level::block::state::block_behaviour::{BlockBehaviour, BlockStateBase};
use crate::level::block::state::state_holder::{StateHolder, StateHolderData};
/// A lightweight BlockState placeholder: holds the "owner" block name and
/// a list of property keys/values (both stored as strings for now).

#[derive(Clone, Debug, PartialEq)]
pub struct BlockState {
    /// Underlying holder data (owner + property arrays).
    pub holder: StateHolderData,
    /// Optional cached derived data (mirrors Java's BlockStateBase cache).
    pub base: Option<BlockStateBase>,
}

impl BlockState {
    /// Create a new BlockState with given owner and property lists.
    pub fn new(
        owner: impl Into<String>,
        property_keys: Vec<String>,
        property_values: Vec<String>,
    ) -> Self {
        Self {
            holder: StateHolderData::new(owner, property_keys, property_values),
            base: None,
        }
    }

    /// Return a reference to self (matches Java `asState()` semantics).
    pub fn as_state(&self) -> &Self {
        self
    }

    /// A minimal factory to produce a default `BlockState` (air).
    pub fn default() -> Self {
        Self {
            holder: StateHolderData::default(),
            base: None,
        }
    }

    /// Return a simple by-name codec instance for `BlockState`.
    ///
    /// In Java this comes from `BuiltInRegistries.BLOCK.byNameCodec()` and is
    /// then wired into the `BlockState` codec. Here we expose a minimal
    /// two-way helper that encodes the block part to its name and decodes a
    /// name back to a `BlockState` with default properties.
    pub fn by_name_codec() -> ByNameCodec {
        ByNameCodec
    }

    /// Initialize (or refresh) the `BlockStateBase` cache from a `BlockBehaviour`.
    pub fn init_cache(&mut self, behaviour: &BlockBehaviour) {
        let mut b = BlockStateBase::new();
        // initialize fields from behaviour
        b.light_emission = behaviour.get_light_emission();
        b.is_air = behaviour.is_air();
        b.destroy_speed = behaviour.default_destroy_time();
        b.can_occlude = behaviour.has_collision();
        self.base = Some(b);
    }

    /// Ensure cache exists and return a reference to it.
    pub fn ensure_cache(&mut self, behaviour: &BlockBehaviour) -> &BlockStateBase {
        if self.base.is_none() {
            self.init_cache(behaviour);
        }
        // safe: just initialized if None
        self.base.as_ref().unwrap()
    }
}

/// Minimal two-way codec for block name <-> `BlockState` conversion.
///
/// This is intentionally tiny: `encode` returns the block name for an
/// existing `BlockState`; `decode` creates a default `BlockState` for a
/// given name. Later this can be wired into a proper serialization/codec
/// subsystem or replaced by a registry-backed implementation.
#[derive(Copy, Clone, Debug)]
pub struct ByNameCodec;

impl ByNameCodec {
    /// Encode a `BlockState` to its owner name.
    pub fn encode(&self, state: &BlockState) -> String {
        state.holder.owner.to_string()
    }

    /// Decode a block name into a (default) `BlockState`.
    pub fn decode(&self, name: &str) -> BlockState {
        BlockState::new(name, Vec::new(), Vec::new())
    }
}

// Implement the minimal StateHolder trait for BlockState by delegating to
// the embedded `StateHolderData`.
impl StateHolder for BlockState {
    fn get_value_str(&self, property_name: &str) -> Option<&str> {
        self.holder.get_value_str(property_name)
    }

    fn set_value_str(&self, property_name: &str, value: String) -> Self {
        let new_holder = self.holder.set_value_str(property_name, value);
        Self {
            holder: new_holder,
            base: None,
        }
    }

    fn values<'a>(&'a self) -> Box<dyn Iterator<Item = (&'a str, &'a str)> + 'a> {
        self.holder.values()
    }
}
