#[cfg(feature = "parallel")]
mod par;
mod seal;

#[cfg(feature = "parallel")]
pub use par::{ParView, ParViews};

use crate::{component::Component, doc, entity, hlist::define_null, query::filter::Filter};
use seal::{ViewSeal, ViewsSeal};

pub trait View<'a>: Filter + ViewSeal<'a> {}

impl<'a, C> View<'a> for &C where C: Component {}

impl<'a, C> View<'a> for &mut C where C: Component {}

impl<'a, C> View<'a> for Option<&C> where C: Component {}

impl<'a, C> View<'a> for Option<&mut C> where C: Component {}

impl<'a> View<'a> for entity::Identifier {}

define_null!();

pub trait Views<'a>: Filter + ViewsSeal<'a> {}

impl<'a> Views<'a> for Null {}

impl<'a, V, W> Views<'a> for (V, W)
where
    V: View<'a>,
    W: Views<'a>,
{
}

doc::non_root_macro! {
    /// Creates a set of [`View`]s over components.
    ///
    /// These views can be used to [`query`] the components stored within a [`World`]. They can also be
    /// used when defining [`System`]s to be run over components stored in a [`World`].
    ///
    /// See the documentation for [`View`] to learn more about what kinds of `View`s can be created.
    ///
    /// # Example
    /// ``` rust
    /// use brood::query::views;
    ///
    /// // Define components.
    /// struct Foo(u32);
    /// struct Bar(bool);
    ///
    /// type Views<'a> = views!(&'a mut Foo, &'a Bar);
    /// ```
    ///
    /// Note that the lifetime `'a` can often be omitted when [`query`]ing a [`World`], but is required
    /// when defining a [`System`].
    ///
    /// [`query`]: crate::world::World::query()
    /// [`System`]: crate::system::System
    /// [`View`]: crate::query::view::View
    /// [`World`]: crate::world::World
    macro_rules! views {
        ($view:ty $(,$views:ty)* $(,)?) => (
            ($view, $crate::views!($($views,)*))
        );
        () => (
            $crate::query::view::Null
        );
    }
}
