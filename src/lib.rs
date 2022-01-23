#![no_std]

extern crate alloc;

pub mod component;
pub mod entity;
pub mod world;

#[doc(hidden)]
pub mod reexports;

mod archetype;
mod archetypes;
mod internal;
mod public;

pub use public::*;
pub use world::World;
