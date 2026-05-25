use std::any::{Any, TypeId};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::packs::resources::resource_manager::ResourceManager;

/// Java 对照: PreparableReloadListener.StateKey<T>
pub struct StateKey<T>(PhantomData<T>);

impl<T> StateKey<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> Default for StateKey<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> fmt::Debug for StateKey<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StateKey")
            .field("type", &std::any::type_name::<T>())
            .finish()
    }
}

/// Java 对照: PreparableReloadListener.SharedState
///
/// Generic over the concrete `ResourceManager` type (e.g.,
/// `MultiPackResourceManager`) so listeners can call any
/// `ResourceManager` method directly.
pub struct SharedState<'a, M: ResourceManager> {
    manager: &'a M,
    state: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl<'a, M: ResourceManager> SharedState<'a, M> {
    pub fn new(manager: &'a M) -> Self {
        Self {
            manager,
            state: HashMap::new(),
        }
    }

    pub fn resource_manager(&self) -> &M {
        self.manager
    }

    pub fn set<T: Send + Sync + 'static>(&mut self, _key: &StateKey<T>, value: T) {
        self.state.insert(TypeId::of::<T>(), Box::new(value));
    }

    pub fn get<T: Send + Sync + 'static>(&self, _key: &StateKey<T>) -> &T {
        self.state
            .get(&TypeId::of::<T>())
            .and_then(|b| b.downcast_ref::<T>())
            .expect("StateKey value not found — did you forget to set()?")
    }
}

/// Java 对照: PreparableReloadListener
///
/// Generic over the concrete `ResourceManager` type so listeners can
/// call any `ResourceManager` method directly (not just
/// `ResourceProvider`).
pub trait PreparableReloadListener<M: ResourceManager>: Send + Sync {
    /// Load / prepare data.  Runs in parallel (scoped threads).
    fn prepare(&self, manager: &M) -> Box<dyn Any + Send>;

    /// Apply prepared data.  Runs sequentially after all prepares.
    fn apply(&self, data: Box<dyn Any + Send>, manager: &M);

    /// Called synchronously before all `reload()` calls to set up
    /// shared state.  Default is a no-op.
    fn prepare_shared_state(&self, _state: &mut SharedState<'_, M>) {}

    /// Human-readable name for diagnostics.
    fn name(&self) -> Cow<'static, str>;
}

// ── Convenience constructors ────────────────────────────────────

/// Java 对照: ResourceManagerReloadListener
///
/// Creates a listener with no preparation — the callback is called
/// during the apply phase.
pub fn resource_manager_listener<M, F>(
    name: impl Into<Cow<'static, str>>,
    callback: F,
) -> impl PreparableReloadListener<M>
where
    M: ResourceManager,
    F: Fn(&M) + Send + Sync + 'static,
{
    ResourceManagerListener {
        name: name.into(),
        callback: Arc::new(callback),
        _marker: PhantomData,
    }
}

struct ResourceManagerListener<M, F> {
    name: Cow<'static, str>,
    callback: Arc<F>,
    _marker: PhantomData<M>,
}

impl<M, F> PreparableReloadListener<M> for ResourceManagerListener<M, F>
where
    M: ResourceManager,
    F: Fn(&M) + Send + Sync + 'static,
{
    fn prepare(&self, _manager: &M) -> Box<dyn Any + Send> {
        Box::new(())
    }

    fn apply(&self, _data: Box<dyn Any + Send>, manager: &M) {
        (self.callback)(manager);
    }

    fn name(&self) -> Cow<'static, str> {
        self.name.clone()
    }
}

/// Java 对照: SimplePreparableReloadListener<T>
///
/// Creates a listener with a separate `prepare` and `apply` phase.
pub fn simple_listener<M, T, Prep, Apply>(
    name: impl Into<Cow<'static, str>>,
    prepare: Prep,
    apply: Apply,
) -> impl PreparableReloadListener<M>
where
    M: ResourceManager,
    T: Send + Sync + 'static,
    Prep: Fn(&M) -> T + Send + Sync + 'static,
    Apply: Fn(T, &M) + Send + Sync + 'static,
{
    SimpleListener {
        name: name.into(),
        prepare: Arc::new(prepare),
        apply: Arc::new(apply),
        _marker: PhantomData,
    }
}

struct SimpleListener<M, T, Prep, Apply> {
    name: Cow<'static, str>,
    prepare: Arc<Prep>,
    apply: Arc<Apply>,
    _marker: PhantomData<(M, T)>,
}

impl<M, T, Prep, Apply> PreparableReloadListener<M>
    for SimpleListener<M, T, Prep, Apply>
where
    M: ResourceManager,
    T: Send + Sync + 'static,
    Prep: Fn(&M) -> T + Send + Sync + 'static,
    Apply: Fn(T, &M) + Send + Sync + 'static,
{
    fn prepare(&self, manager: &M) -> Box<dyn Any + Send> {
        Box::new((self.prepare)(manager))
    }

    fn apply(&self, data: Box<dyn Any + Send>, manager: &M) {
        let value = *data
            .downcast::<T>()
            .expect("type mismatch in SimpleListener::apply");
        (self.apply)(value, manager);
    }

    fn name(&self) -> Cow<'static, str> {
        self.name.clone()
    }
}

