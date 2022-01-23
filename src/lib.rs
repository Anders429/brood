#![no_std]

extern crate alloc;

#[doc(hidden)]
pub mod reexports;

mod archetype;
mod internal;
mod public;

pub use public::*;
