#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc(
    html_logo_url = "https://bevyengine.org/assets/icon.png",
    html_favicon_url = "https://bevyengine.org/assets/icon.png"
)]
#![no_std]

/// Provides the state of features in this crate.
pub mod cfg {
    pub use bevy_platform::cfg::*;

    define_alias! {
        all(
            not(target_arch = "wasm32"),
            feature = "multi_threaded",
            any(
                feature = "futures-lite-block-on",
                feature = "async-io"
            )
        ) => {
            /// Enables multi-threading support.
            /// Without this feature, all tasks will be run on a single thread.
            multi_threaded
        }
        not(all(
            not(target_arch = "wasm32"),
            feature = "multi_threaded",
            any(
                feature = "futures-lite-block-on",
                feature = "async-io"
            )
        )) => {
            /// Compiling in single-threaded mode.
            single_threaded
        }
        feature = "async_executor" => {
            /// Uses `async-executor` as a task execution backend.
            /// This backend is incompatible with `no_std` targets.
            async_executor
        }
        feature = "async-io" => {
            /// Uses `async-io` for asynchronous IO integration.
            async_io
        }
        feature = "futures-lite-block-on" => {
            /// Re-exports the `block_on` function from `futures-lite` if `async-io` is not enabled.
            futures_lite_block_on
        }
        not(feature = "async_executor") => {
            /// Uses an internal fork of `edge-executor` as the execution backend.
            /// This backend is compatible with `no_std` targets.
            edge_executor
        }
        any(feature = "futures-lite-block-on", feature = "async-io") => {
            /// Provides an implementation for `block_on`.
            block_on
        }
        all(target_arch = "wasm32", feature = "web-event-loop") => {
            /// Integrates with the web browser event loop for ticking pending tasks.
            web_event_loop
        }
    }
}

cfg::std! {
    extern crate std;
}

extern crate alloc;

cfg::switch! {
    #[cfg(target_arch = "wasm32")] => {
        /// Use [`ConditionalSend`] to mark an optional Send trait bound. Useful as on certain platforms (eg. Wasm),
        /// futures aren't Send.
        pub trait ConditionalSend {}
        impl<T> ConditionalSend for T {}
    }
    _ => {
        /// Use [`ConditionalSend`] to mark an optional Send trait bound. Useful as on certain platforms (eg. Wasm),
        /// futures aren't Send.
        pub trait ConditionalSend: Send {}
        impl<T: Send> ConditionalSend for T {}
    }
}

/// Use [`ConditionalSendFuture`] for a future with an optional Send trait bound, as on certain platforms (eg. Wasm),
/// futures aren't Send.
pub trait ConditionalSendFuture: Future + ConditionalSend {}
impl<T: Future + ConditionalSend> ConditionalSendFuture for T {}

use alloc::boxed::Box;

/// An owned and dynamically typed Future used when you can't statically type your result or need to add some indirection.
pub type BoxedFuture<'a, T> = core::pin::Pin<Box<dyn ConditionalSendFuture<Output = T> + 'a>>;

pub mod futures;

cfg::edge_executor! {
    mod edge_executor;
}

mod executor;

mod slice;
pub use slice::{ParallelSlice, ParallelSliceMut};

cfg::switch! {
    cfg::web_event_loop => {
        mod wasm_task as task;
    }
    _ => {
        mod task;
    }
}

pub use task::Task;

cfg::switch! {
    cfg::multi_threaded => {
        mod task_pool;
        mod thread_executor;

        pub use task_pool::{Scope, TaskPool, TaskPoolBuilder};
        pub use thread_executor::{ThreadExecutor, ThreadExecutorTicker};
    }
    _ => {
        mod single_threaded_task_pool;

        pub use single_threaded_task_pool::{Scope, TaskPool, TaskPoolBuilder, ThreadExecutor};
    }
}

mod usages;
pub use futures_lite::future::poll_once;
pub use usages::{
    tick_global_task_pools_on_main_thread, AsyncComputeTaskPool, ComputeTaskPool, IoTaskPool,
};

cfg::switch! {
    cfg::async_io => {
        pub use async_io::block_on;
    }
    cfg::futures_lite_block_on => {
        pub use futures_lite::future::block_on;
    }
}

mod iter;
pub use iter::ParallelIterator;

pub use futures_lite;

/// The tasks prelude.
///
/// This includes the most common types in this crate, re-exported for your convenience.
pub mod prelude {
    #[doc(hidden)]
    pub use crate::{
        iter::ParallelIterator,
        slice::{ParallelSlice, ParallelSliceMut},
        usages::{AsyncComputeTaskPool, ComputeTaskPool, IoTaskPool},
    };

    crate::cfg::block_on! {
        #[doc(hidden)]
        pub use crate::block_on;
    }
}

cfg::switch! {
    cfg::std => {
        use core::num::NonZero;

        /// Gets the logical CPU core count available to the current process.
        ///
        /// This is identical to [`std::thread::available_parallelism`], except
        /// it will return a default value of 1 if it internally errors out.
        ///
        /// This will always return at least 1.
        pub fn available_parallelism() -> usize {
            std::thread::available_parallelism()
                .map(NonZero::<usize>::get)
                .unwrap_or(1)
        }
    }
    _ => {
        /// Gets the logical CPU core count available to the current process.
        ///
        /// This will always return at least 1.
        pub fn available_parallelism() -> usize {
            // Without access to std, assume a single thread is available
            1
        }
    }
}
