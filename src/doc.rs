//! Documentation-specific tools.
//!
//! These are items specific to this crate's documentation. Nothing contained in this module should
//! affect how this crate works logically; things included here should only affect how the crate is
//! documented.

/// Defines a macro that is not documented at the crate's root.
///
/// This changes the way the macro is defined only when the `doc_cfg` cfg flag is passed. It
/// utilizes the unstable `decl_macro` nightly feature to define the macro in-place, rather than
/// defining it at the root level of the crate.
macro_rules! non_root_macro {
    (
        $(#[$m:meta])*
        macro_rules! $name:ident {
            $(($($token:tt)*) => ($($expansion:tt)*));* $(;)?
        }
    ) => (
        $(#[$m])*
        #[doc(hidden)]
        #[cfg(not(doc_cfg))]
        #[macro_export]
        macro_rules! $name {
            $(
                ($($token)*) => ($($expansion)*)
            );*
        }

        $(#[$m])*
        #[cfg(not(doc_cfg))]
        pub use $name;

        $(#[$m])*
        #[cfg(doc_cfg)]
        pub macro $name {
            $(
                ($($token)*) => ($($expansion)*)
            ),*
        }
    )
}

pub(crate) use non_root_macro;
