use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::packs::resources::preparable_reload_listener::{
    PreparableReloadListener, SharedState,
};
use crate::packs::resources::reload_instance::ReloadInstance;
use crate::packs::resources::resource_manager::ResourceManager;

/// Java 对照: SimpleReloadInstance.create / SimpleReloadInstance.of
///
/// Kicks off a full resource reload on a background thread.
///
/// 1. `prepare_shared_state` for all listeners (synchronous)
/// 2. `prepare` for all listeners in **parallel** (scoped threads)
/// 3. `apply` for all listeners **sequentially** on the background thread
pub fn create_reload<M: ResourceManager + 'static>(
    manager: Arc<M>,
    listeners: Vec<Arc<dyn PreparableReloadListener<M>>>,
) -> ReloadInstance {
    let total = listeners.len() as u32;
    let progress = Arc::new(AtomicU32::new(0));
    let progress_clone = Arc::clone(&progress);

    let handle = thread::spawn(move || {
        // Step 1: shared state
        let mut shared_state = SharedState::new(&*manager);
        for listener in &listeners {
            listener.prepare_shared_state(&mut shared_state);
        }

        // Step 2: parallel preparation
        let prep_results: Arc<Mutex<Vec<Option<Box<dyn std::any::Any + Send>>>>> =
            Arc::new(Mutex::new((0..total).map(|_| None).collect()));

        thread::scope(|s| {
            for (i, listener) in listeners.iter().enumerate() {
                let results = Arc::clone(&prep_results);
                let mgr = Arc::clone(&manager);
                s.spawn(move || {
                    let data = listener.prepare(&*mgr);
                    results.lock().unwrap()[i] = Some(data);
                });
            }
        });

        // Step 3: sequential application
        let mut prep_results = Arc::into_inner(prep_results)
            .expect("all scoped threads joined")
            .into_inner()
            .unwrap();

        for (listener, data) in listeners
            .iter()
            .zip(prep_results.drain(..).map(|o| {
                o.expect("prepare did not produce a result")
            }))
        {
            listener.apply(data, &*manager);
            progress_clone.fetch_add(1, Ordering::SeqCst);
        }
    });

    ReloadInstance::new(handle, progress, total)
}
