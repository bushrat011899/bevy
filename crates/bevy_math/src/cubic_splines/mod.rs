//! Provides types for building cubic splines for rendering curves and use with animation easing.

mod segments;
pub use segments::*;

crate::cfg::alloc! {
    mod splines;
    pub use splines::*;
}

crate::cfg::curve! {
    mod curve_impls;
}
