#![no_std]

extern crate alloc;

pub mod component;
pub mod entities;
pub mod entity;
pub mod registry;
pub mod system;
pub mod world;

#[doc(hidden)]
pub mod reexports;

mod archetype;
mod archetypes;
mod internal;
mod public;

pub use public::*;
pub use world::World;
