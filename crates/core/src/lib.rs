#![doc = include_str!(concat!("../", env!("CARGO_PKG_README")))]

/// Trait for composable mock data.
///
/// Similar to [`std::default::Default`], but for tests.
pub trait Mock: Sized {
    /// Returns `Self` containing a non-random test value
    fn mock() -> Self;
}

impl<T: Mock> Mock for Option<T> {
    fn mock() -> Self {
        Some(Mock::mock())
    }
}

impl Mock for () {
    fn mock() -> Self {}
}

pub use damock_macros::Mock;
