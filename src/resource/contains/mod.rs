mod resource;
mod views;

pub use resource::ContainsResource;
pub use views::ContainsViews;

use crate::hlist::define_null_uninstantiable;

define_null_uninstantiable!();

pub enum Contained {}
pub enum NotContained {}
