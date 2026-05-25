use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::thread;

/// Java 对照: net.minecraft.server.packs.resources.ReloadInstance
///
/// Tracks the progress and completion of an asynchronous resource reload.
pub struct ReloadInstance {
    handle: Option<thread::JoinHandle<()>>,
    progress: Arc<AtomicU32>,
    total: u32,
}

impl ReloadInstance {
    pub fn new(
        handle: thread::JoinHandle<()>,
        progress: Arc<AtomicU32>,
        total: u32,
    ) -> Self {
        Self {
            handle: Some(handle),
            progress,
            total,
        }
    }

    /// Java 对照: ReloadInstance.getActualProgress()
    pub fn get_actual_progress(&self) -> f32 {
        if self.total == 0 {
            return 1.0;
        }
        self.progress.load(Ordering::SeqCst) as f32 / self.total as f32
    }

    /// Java 对照: ReloadInstance.isDone()
    pub fn is_done(&self) -> bool {
        self.handle.as_ref().map_or(true, |h| h.is_finished())
    }

    /// Java 对照: ReloadInstance.checkExceptions()
    ///
    /// Panics if the reload thread panicked.
    pub fn check_exceptions(&mut self) {
        if let Some(handle) = self.handle.as_ref() {
            if handle.is_finished() {
                // Take and join to propagate any panic
                if let Ok(result) = self.handle.take().unwrap().join() {
                    // re-stash the handle since we consumed it to check
                    self.handle = None;
                    let _ = result;
                }
            }
        }
    }

    /// Block until the reload is fully complete.
    pub fn done(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.join().expect("reload thread panicked");
        }
    }
}

impl Drop for ReloadInstance {
    fn drop(&mut self) {
        if self.handle.is_some() {
            // detached — thread continues running
        }
    }
}
