//! Provides various atomic alternatives to language primitives.
//!
//! Certain platforms lack complete atomic support, requiring the use of a fallback
//! such as `portable-atomic`.
//! Using these types will ensure the correct atomic provider is used without the need for
//! feature gates in your own code.

pub use atomic_16::{AtomicI16, AtomicU16};
pub use atomic_32::{AtomicI32, AtomicU32};
pub use atomic_64::{AtomicI64, AtomicU64};
pub use atomic_8::{AtomicBool, AtomicI8, AtomicU8};
pub use atomic_ptr::{AtomicIsize, AtomicPtr, AtomicUsize};
pub use core::sync::atomic::Ordering;

use crate::cfg::switch;

switch! {
    target_has_atomic = "8" => {
        use core::sync::atomic as atomic_8;
    }
    _ => {
        use portable_atomic as atomic_8;
    }
}

switch! {
    target_has_atomic = "16" => {
        use core::sync::atomic as atomic_16;
    }
    _ => {
        use portable_atomic as atomic_16;
    }
}

switch! {
    target_has_atomic = "32" => {
        use core::sync::atomic as atomic_32;
    }
    _ => {
        use portable_atomic as atomic_32;
    }
}

switch! {
    target_has_atomic = "64" => {
        use core::sync::atomic as atomic_64;
    }
    _ => {
        use portable_atomic as atomic_64;
    }
}

switch! {
    target_has_atomic = "ptr" => {
        use core::sync::atomic as atomic_ptr;
    }
    _ => {
        use portable_atomic as atomic_ptr;
    }
}
