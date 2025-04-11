//! Provides `LazyLock`

pub use implementation::LazyLock;

crate::cfg::std!(if {
    use std::sync as implementation;
} else {
    mod implementation {
        pub use spin::Lazy as LazyLock;
    }
});
