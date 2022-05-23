#![no_std]
#![cfg_attr(doc_cfg, feature(doc_cfg, decl_macro))]
#![warn(
    clippy::pedantic,
    clippy::undocumented_unsafe_blocks,
    unsafe_op_in_unsafe_fn
)]
#![allow(clippy::module_name_repetitions)]

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
mod r#macro;

#[doc(inline)]
pub use world::World;
