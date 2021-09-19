#![cfg_attr(doc, feature(doc_cfg))]
#![no_std]

extern crate alloc;

#[doc(hidden)]
pub mod reexports;

mod internal;
mod public;

pub use public::*;
