//! Documentation-specific tools.
//!
//! These are items specific to this crate's documentation. Nothing contained in this module should
//! affect how this crate works logically; things included here should only affect how the crate is
//! documented.

/// Wrapper around a token tree to ensure it is not parsed at compile time if it is excluded by
/// `cfg` rules.
///
/// This is necessary for using the `macro` keyword for controlling where macro documentation is
/// exported, as the `macro` syntax is experimental and subject to change at any time. This
/// protects the crate from being unusable if that syntax changes, as the changes would cause all
/// compilation, even compilation not enabling `doc_cfg`, to break otherwise.
#[cfg(doc_cfg)]
macro_rules! unparsed {
    ( $($tokens:tt)* ) => { $($tokens)* }
}

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

        #[cfg(doc_cfg)]
        crate::doc::unparsed! {
            $(#[$m])*
            pub macro $name {
                $(
                    ($($token)*) => ($($expansion)*)
                ),*
            }
        }
    )
}

pub(crate) use non_root_macro;
#[cfg(doc_cfg)]
pub(crate) use unparsed;
