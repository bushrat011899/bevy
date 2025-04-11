//! Provides `Instant` for all platforms.

pub use time::Instant;

use crate::cfg;

cfg::switch! {
    cfg::web => {
        use web_time as time;
    }
    cfg::std => {
        use std::time;
    }
    _ => {
        mod fallback;

        use fallback as time;
    }
}
