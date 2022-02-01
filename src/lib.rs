#![no_std]
#![cfg_attr(doc_cfg, feature(doc_cfg, decl_macro))]

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
mod doc;
mod hlist;

#[doc(inline)]
pub use world::World;
