mod seal;

pub(crate) use seal::{
    ParViewsSeal,
    RepeatNone,
};

use crate::{
    component,
    entity,
    query::view::Null,
};
use seal::ParViewSeal;

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
/// [`Component`]: crate::component::Component
/// [`ParSystem`]: crate::system::ParSystem
/// [`par_query`]: crate::world::World::par_query()
/// [`View`]: crate::query::view::View
#[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
pub trait ParView<'a>: ParViewSeal<'a> + Send {}

impl<'a, Component> ParView<'a> for &'a Component where Component: component::Component + Sync {}

impl<'a, Component> ParView<'a> for &'a mut Component where Component: component::Component + Send {}

impl<'a, Component> ParView<'a> for Option<&'a Component> where
    Component: component::Component + Sync
{
}

impl<'a, Component> ParView<'a> for Option<&'a mut Component> where
    Component: component::Component + Send
{
}

impl<'a> ParView<'a> for entity::Identifier {}

/// A heterogeneous list of [`ParView`]s.
///
/// The main difference between this trait and the standard [`Views`] trait is that these views can
/// be shared between threads, allowing them to be used within parallel iteration in either a
/// [`ParSystem`] or a [`par_query`].
///
/// All types that implement `Views` also implement `ParViews`, so long as any [`Component`]s `C`
/// they view are [`Send`] when viewed mutably or [`Sync`] when viewed immutably.
///
/// # Example
/// ``` rust
/// use brood::query::Views;
///
/// // Define components.
/// struct Foo(usize);
/// struct Bar(bool);
///
/// // Define views over those components.
/// type Views<'a> = Views!(&'a Foo, &'a mut Bar);
/// ```
///
/// Because the `Component`s viewed above implement both [`Send`] and [`Sync`], the views created
/// above implement `ParViews`.
///
/// [`Component`]: crate::component::Component
/// [`ParSystem`]: crate::system::ParSystem
/// [`ParView`]: crate::query::view::ParView
/// [`par_query`]: crate::world::World::par_query()
/// [`Views`]: trait@crate::query::view::Views
#[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
pub trait ParViews<'a>: ParViewsSeal<'a> + Send {}

impl<'a> ParViews<'a> for Null {}

impl<'a, ParView, ParViews> self::ParViews<'a> for (ParView, ParViews)
where
    ParView: self::ParView<'a>,
    ParViews: self::ParViews<'a>,
{
}
