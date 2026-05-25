//! Java 对照: net.minecraft.server.packs.resources.ResourceManagerReloadListener
//!
//! Rust 里不存在对应的子接口。请使用 `resource_manager_listener` 构造函数替代：
//!
//! ```ignore
//! use server::packs::resources::preparable_reload_listener::resource_manager_listener;
//!
//! mgr.register_reload_listener(Box::new(resource_manager_listener(
//!     "my_listener",
//!     |manager: &MultiPackResourceManager| {
//!         // reload logic here
//!     },
//! )));
//! ```
//!
//! Java 中 `SimplePreparableReloadListener<T>` 对应 `simple_listener` 函数。
