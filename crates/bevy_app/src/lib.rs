#![cfg_attr(
    any(docsrs, docsrs_dep),
    expect(
        internal_features,
        reason = "rustdoc_internals is needed for fake_variadic"
    )
)]
#![cfg_attr(any(docsrs, docsrs_dep), feature(doc_auto_cfg, rustdoc_internals))]
#![forbid(unsafe_code)]
#![doc(
    html_logo_url = "https://bevyengine.org/assets/icon.png",
    html_favicon_url = "https://bevyengine.org/assets/icon.png"
)]
#![no_std]

//! This crate is about everything concerning the highest-level, application layer of a Bevy app.

/// Provides the state of features in this crate.
pub mod cfg {
    pub use bevy_ecs::cfg::bevy_debug_stepping;
    pub use bevy_platform::cfg::*;

    define_alias! {
        all(any(unix, windows), feature = "ctrl_c") => {
            /// Integrates with a Ctrl-C signal sent in the terminal.
            /// Only applicable on Windows and Unix platforms.
            ctrl_c
        }
        feature = "error_panic_hook" => {
            /// Will set the BevyError panic hook, which gives cleaner filtered backtraces when
            /// a BevyError is hit.
            error_panic_hook
        }
        all(target_arch = "wasm32", feature = "web_panic_hook") => {
            /// Integrates with web browser APIs to forward panics to the console logger.
            web_panic_hook
        }
        feature = "bevy_reflect" => {
            /// Adds runtime reflection support using `bevy_reflect`.
            bevy_reflect
        }
        all(target_arch = "wasm32", feature = "web") => {
            /// Enables use of browser APIs.
            web
        }
        feature = "trace" => {
            /// Enables `tracing` integration, allowing spans and other metrics to be reported
            /// through that framework.
            trace
        }
    }

    // Always ensure reflect_functions is available, even if bevy_reflect isn't included.
    switch! {
        bevy_reflect => {
            #[doc(inline)]
            pub use bevy_reflect::cfg::functions as reflect_functions;
        }
        _ => {
            #[doc(inline)]
            pub use noop as reflect_functions;
        }
    }
}

cfg::std! {
    extern crate std;
}

extern crate alloc;

// Required to make proc macros work in bevy itself.
extern crate self as bevy_app;

mod app;
mod main_schedule;
mod panic_handler;
mod plugin;
mod plugin_group;
mod schedule_runner;
mod sub_app;
mod task_pool_plugin;

pub use app::*;
pub use main_schedule::*;
pub use panic_handler::*;
pub use plugin::*;
pub use plugin_group::*;
pub use schedule_runner::*;
pub use sub_app::*;
pub use task_pool_plugin::*;

cfg::ctrl_c! {
    mod terminal_ctrl_c_handler;
    pub use terminal_ctrl_c_handler::*;
}

/// The app prelude.
///
/// This includes the most common types in this crate, re-exported for your convenience.
pub mod prelude {
    #[doc(hidden)]
    pub use crate::{
        app::{App, AppExit},
        main_schedule::{
            First, FixedFirst, FixedLast, FixedPostUpdate, FixedPreUpdate, FixedUpdate, Last, Main,
            PostStartup, PostUpdate, PreStartup, PreUpdate, RunFixedMainLoop,
            RunFixedMainLoopSystem, SpawnScene, Startup, Update,
        },
        sub_app::SubApp,
        Plugin, PluginGroup, TaskPoolOptions, TaskPoolPlugin,
    };
}
