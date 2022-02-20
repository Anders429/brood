//! Helper macros specifically for use by other macros within this crate.
//!
//! Although the macros here are exported publicly, it is only for use by other macros within this
//! crate. These should not be depended upon by any external code, and are not considered a part of
//! the public API.

/// Indicate that the given tokens were unexpected.
///
/// Calling this macro with any tokens will return an error pointing to those tokens within the
/// original macro. This allows more precise error messages from macros when input is not formatted
/// correctly.
#[macro_export]
#[doc(hidden)]
macro_rules! unexpected {
    () => {};
}
