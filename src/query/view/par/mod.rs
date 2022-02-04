mod seal;

use crate::{
    component::Component,
    entity,
    query::{filter::Filter, view::Null},
};
use seal::{ParViewSeal, ParViewsSeal};

/// A parallel view over a single aspect of an entity.
///
/// The main difference between this trait and the standard [`View`] trait is that this view can be
/// shared between threads, allowing it to be used within parallel iteration in either a
/// [`ParSystem`] or a [`par_query`].
///
/// All types that implement `View` also implement `ParView`, so long as any [`Component`] `C` they
/// view is [`Send`] when viewed mutably or [`Sync`] when viewed immutably.
///
/// # Example
/// ``` rust
/// // Define a component.
/// struct Foo(usize);
///
/// // Define a view over that component.
/// type FooView<'a> = &'a Foo;
/// ```
///
/// Because the `Component` viewed in the above example implements `Sync`, the view created above
/// implements `ParView`.
///
/// [`ParSystem`]: crate::system::ParSystem
/// [`par_query`]: crate::world::World::par_query()
/// [`View`]: crate::query::view::View
#[cfg_attr(doc_cfg, doc(cfg(feature = "parallel")))]
pub trait ParView<'a>: Filter + ParViewSeal<'a> {}

impl<'a, C> ParView<'a> for &C where C: Component + Sync {}

impl<'a, C> ParView<'a> for &mut C where C: Component + Send {}

impl<'a, C> ParView<'a> for Option<&C> where C: Component + Sync {}

impl<'a, C> ParView<'a> for Option<&mut C> where C: Component + Send {}

impl<'a> ParView<'a> for entity::Identifier {}

#[cfg_attr(doc_cfg, doc(cfg(feature = "parallel")))]
pub trait ParViews<'a>: Filter + ParViewsSeal<'a> {}

impl<'a> ParViews<'a> for Null {}

impl<'a, V, W> ParViews<'a> for (V, W)
where
    V: ParView<'a>,
    W: ParViews<'a>,
{
}
