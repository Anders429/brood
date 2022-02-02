mod seal;

use crate::{doc, hlist::define_null, system::{schedule::task::Task, ParSystem, System}};
use seal::Seal;

pub enum Stage<S, P> {
    Start(Task<S, P>),
    Continue(Task<S, P>),
    Flush,
}

pub trait Stages<'a>: Seal<'a> {}

define_null!();

impl<'a> Stages<'a> for Null {}

impl<'a, S, P, L> Stages<'a> for (Stage<S, P>, L)
where
    S: System<'a> + Send,
    P: ParSystem<'a> + Send,
    L: Stages<'a>,
{
}

doc::non_root_macro! {
    /// Define type annotations for stages wihtin a [`Schedule`].
    ///
    /// This macro removes all the boilerplate involved in creating a type annotation for a
    /// `Schedule` by allowing an easy definition of stages the `Schedule` is generic over.
    ///
    /// There are three different types of commands that can be given to a `stages!` definition.
    /// These match with the commands that can be used to create a `Schedule` using a
    /// [`schedule::Builder`]. They are:
    ///
    /// - `system: <system>`: runs a [`System`].
    /// - `par_system: <par_system>`: runs a [`ParSystem`].
    /// - `flush`: waits for completion of current stage and runs post-processing before proceeding.
    ///
    /// These can be provided to the macro to generate the correct type annotations, like so:
    ///
    /// ``` rust
    /// use brood::{query::{filter, result, views}, registry::Registry, system::{schedule::stages, System, ParSystem}};
    ///
    /// // Define components.
    /// struct A;
    /// struct B;
    /// struct C;
    ///
    /// // Define a System.
    /// struct Foo;
    /// impl<'a> System<'a> for Foo {
    ///     type Filter = filter::None;
    ///     type Views = views!(&'a mut A, &'a B);
    ///
    ///     fn run<R>(&mut self, query_results: result::Iter<'a, R, Self::Filter, Self::Views>)
    ///     where
    ///         R: Registry + 'a,
    ///     {
    ///         // Operate on result here.
    ///     }
    /// }
    /// 
    /// // Define a Parallel System.
    /// struct Bar;
    /// impl<'a> ParSystem<'a> for Bar {
    ///     type Filter = filter::None;
    ///     type Views = views!(&'a B, &'a mut C);
    ///
    ///     fn run<R>(&mut self, query_results: result::ParIter<'a, R, Self::Filter, Self::Views>)
    ///     where
    ///         R: Registry + 'a
    ///     {
    ///         // Operate on result here.
    ///     }
    /// }
    ///
    /// type Stages = stages!{
    ///     system: Foo,
    ///     flush,
    ///     par_system: Bar,
    /// };
    /// ```
    ///
    /// The above example will create stages operating the system `Foo`, followed by performing 
    /// post-processing, and then run the parallel system `Bar`.
    ///
    /// [`ParSystem`]: crate::system::ParSystem
    /// [`Schedule`]: crate::system::Schedule
    /// [`schedule::Builder`]: crate::system::schedule::Builder
    /// [`System`]: crate::system::System
    macro_rules! stages {
        ($($idents:tt $(: $systems:tt)?),* $(,)?) => (
            stages!(internal @ $crate::system::schedule::stage::Null; $($idents $(: $systems)?,)*)
        );
        (internal @ $processed:ty; system: $system:ty, $($idents:tt $(: $systems:tt)?),* $(,)?) => (
            stages!(internal @ ($crate::system::schedule::stage::Stage<$system, $crate::system::Null>, $processed); $($idents $(: $systems)?,)*)
        );
        (internal @ $processed:ty; par_system: $par_system:ty, $($idents:tt $(: $systems:tt)?),* $(,)?) => (
            stages!(internal @ ($crate::system::schedule::stage::Stage<$crate::system::Null, $par_system>, $processed); $($idents $(: $systems)?,)*)
        );
        (internal @ $processed:ty; flush, $($idents:tt $(: $systems:tt)?),* $(,)?) => (
            stages!(internal @ ($crate::system::schedule::stage::Stage<$crate::system::Null, $crate::system::Null>, $processed); $($idents $(: $systems)?,)*)
        );
        (internal @ $processed:ty; $($idents:tt $(: $systems:tt)?),* $(,)?) => (
            $processed
        );
    }
}
