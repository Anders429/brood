#![no_std]

extern crate alloc;

pub mod component;
pub mod entities;
pub mod entity;
pub mod query;
pub mod registry;
pub mod system;
pub mod world;

#[doc(hidden)]
pub mod reexports;

mod archetype;
mod archetypes;

pub use world::World;
