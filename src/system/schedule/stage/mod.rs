//! Tasks scheduled in stages.
//!
//! [`Stages`] are [`System`]s that are scheduled to run in order, with parallelizable stages
//! already grouped together. These are created by a [`schedule::Builder`]. User-facing code will
//! not normally need to create `Stages` outside of the `schedule::Builder`. However, if a defined
//! type of a `Stages` is needed, a [`stages!`] macro is provided to easily define the type.
//!
//! [`schedule::Builder`]: crate::system::schedule::Builder
//! [`Stages`]: crate::system::schedule::stage::Stages
//! [`stages!`]: crate::system::schedule::stages!
//! [`System`]: crate::system::System

mod seal;

use crate::{
    doc,
    hlist::define_null,
    query::filter::Filter,
    registry::{ContainsParViews, ContainsViews, Registry},
    system::{schedule::task::Task, ParSystem, System},
};
use seal::Seal;

/// A single step in a stage.
///
/// A step is a single task in a stage, along with information about whether this task is the
/// beginning of a new stage or if it is a continuation of the current stage.
pub enum Stage<S, P> {
    Start(Task<S, P>),
    Continue(Task<S, P>),
    Flush,
}

/// A heterogeneous list of [`Stage`]s.
///
/// The ordered `Stage`s provided here define the actual stages of the schedule. Note that the
/// stages are defined inside-out, with the last of the heterogeneous list being the beginning of
/// the list of stages.
pub trait Stages<'a, R, SFI, SVI, PFI, PVI, SP, SI, PP, PI>:
    Seal<'a, R, SFI, SVI, PFI, PVI, SP, SI, PP, PI>
where
    R: Registry + 'a,
{
}

define_null!();

impl<'a, R> Stages<'a, R, Null, Null, Null, Null, Null, Null, Null, Null> for Null where
    R: Registry + 'a
{
}

impl<
        'a,
        S,
        P,
        L,
        R,
        SFI,
        SFIS,
        SVI,
        SVIS,
        PFI,
        PFIS,
        PVI,
        PVIS,
        SP,
        SPS,
        SI,
        SIS,
        PP,
        PPS,
        PI,
        PIS,
    >
    Stages<
        'a,
        R,
        (SFI, SFIS),
        (SVI, SVIS),
        (PFI, PFIS),
        (PVI, PVIS),
        (SP, SPS),
        (SI, SIS),
        (PP, PPS),
        (PI, PIS),
    > for (Stage<S, P>, L)
where
    R: Registry + 'a,
    R::Viewable: ContainsViews<'a, S::Views, SP, SI> + ContainsParViews<'a, P::Views, PP, PI>,
    S: System<'a> + Send,
    S::Filter: Filter<R, SFI>,
    S::Views: Filter<R, SVI>,
    P::Filter: Filter<R, PFI>,
    P::Views: Filter<R, PVI>,
    P: ParSystem<'a> + Send,
    L: Stages<'a, R, SFIS, SVIS, PFIS, PVIS, SPS, SIS, PPS, PIS>,
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
    /// use brood::{query::{filter, filter::Filter, result, views}, registry::{ContainsParViews, ContainsViews, Registry}, system::{schedule::stages, System, ParSystem}};
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
    ///     fn run<R, FI, VI, P, I>(&mut self, query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views, VI>)
    ///     where
    ///         R: Registry + 'a,
    ///         R::Viewable: ContainsViews<'a, Self::Views, P, I>,
    ///         Self::Filter: Filter<R, FI>,
    ///         Self::Views: Filter<R, VI>,
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
    ///     fn run<R, FI, VI, P, I>(&mut self, query_results: result::ParIter<'a, R, Self::Filter, FI, Self::Views, VI>)
    ///     where
    ///         R: Registry + 'a,
    ///         R::Viewable: ContainsParViews<'a, Self::Views, P, I>,
    ///         Self::Filter: Filter<R, FI>,
    ///         Self::Views: Filter<R, VI>,
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
        // Entry point for the macro.
        //
        // This just calls into the internal macro with all of the correct parameters.
        ($($tt:tt)*) => (
            $crate::stages_internal!(@task $crate::system::schedule::stage::Null; ($($tt)*) ($($tt)*))
        );
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! stages_internal {
    ///////////////////////////////////////////////////////////////////////////////////////////////
    // Generic task macro.
    //
    // These patterns are for matching and processing generic tasks listed in the token tree. When
    // each of the valid task types (system, par_system, and flush) are encountered, the tokens are
    // passed to the appropriate patterns (@system, @par_system, and @flush) to be consumed.
    //
    // Note that we require two copies of the input token tree here. The first is matched on, and
    // the second is used to trigger errors.
    //
    // Invoked as:
    // $crate::stages_internal!(@task $crate::system::schedule::stage::Null; ($($tt)*) ($($tt)*))
    ///////////////////////////////////////////////////////////////////////////////////////////////

    // Match a system task.
    (@task $processed:ty; (system $($rest:tt)*) $copy:tt) => (
        $crate::stages_internal!(@system $processed; ($($rest)*) ($($rest)*))
    );

    // Match a par_system task.
    (@task $processed:ty; (par_system $($rest:tt)*) $copy:tt) => (
        $crate::stages_internal!(@par_system $processed; ($($rest)*) ($($rest)*))
    );

    // Match a flush task.
    (@task $processed:ty; (flush $($rest:tt)*) $copy:tt) => (
        $crate::stages_internal!(@flush $processed; ($($rest)*) ($($rest)*))
    );

    // End case. We return the processed result.
    (@task $processed:ty; () ()) => (
        $processed
    );

    // Match any other unexpected input. This will output an error.
    (@task $processed:ty; ($($rest:tt)*) $copy:tt) => (
        $crate::unexpected!($($rest)*)
    );

    ///////////////////////////////////////////////////////////////////////////////////////////////
    // System task macro.
    //
    // These patterns match the remainder of a system task entry. This will match a colon followed
    // by a system type. Trailing commas are also matched here.
    //
    // Note that we require two copies of the input token tree here. The first is matched on, and
    // the second is used to trigger errors.
    ///////////////////////////////////////////////////////////////////////////////////////////////

    // Match a system type without a trailing comma. This will only match at the end of the token
    // tree.
    (@system $processed:ty; (: $system:ty) $copy:tt) => {
        $crate::stages_internal(@task ($crate::system::schedule::stage::Stage<$system, $crate::system::Null>, $processed); () ())
    };

    // Match a system type with a trailing comma.
    (@system $processed:ty; (: $system:ty, $($rest:tt)*) $copy:tt) => (
        $crate::stages_internal!(@task ($crate::system::schedule::stage::Stage<$system, $crate::system::Null>, $processed); ($($rest)*) ($($rest)*))
    );

    // Match a system type with no trailing comma not at the end of the token tree. This will
    // output an error.
    (@system $processed:ty; (: $system:tt $($rest:tt)*) $copy:tt) => {
        $crate::unexpected!($($rest)*)
    };

    // Match a comma with no system type provided. This will output an error.
    (@system $processed:ty; (, $($rest:tt)*) ($comma:tt $($copy:tt)*)) => (
        $crate::unexpected!($comma)
    );

    // Match any other unexpected input. This will output an error.
    (@system $processed:ty; ($($rest:tt)*) $copy:tt) => (
        $crate::unexpected!($($rest)*)
    );

    ///////////////////////////////////////////////////////////////////////////////////////////////
    // Parallel system task macro.
    //
    // These patterns match the remainder of a par_system task entry. This will match a colon
    // followed by a parallel system type. Trailing commas are also matched here.
    //
    // Note that we require two copies of the input token tree here. The first is matched on, and
    // the second is used to trigger errors.
    ///////////////////////////////////////////////////////////////////////////////////////////////

    // Match a parallel system type without a trailing comma. This will only match at the end of
    // the token tree.
    (@par_system $processed:ty; (: $par_system:ty) $copy:tt) => (
        $crate::stages_internal!(@task ($crate::system::schedule::stage::Stage<$crate::system::Null, $par_system>, $processed); () ())
    );

    // Match a parallel system type with a trailing comma.
    (@par_system $processed:ty; (: $par_system:ty, $($rest:tt)*) $copy:tt) => (
        $crate::stages_internal!(@task ($crate::system::schedule::stage::Stage<$crate::system::Null, $par_system>, $processed); ($($rest)*) ($($rest)*))
    );

    // Match a parallel system type with no trailing comma not at the end of the token tree. This
    // will output an error.
    (@par_system $processed:ty; (: $par_system:tt $($rest:tt)*) $copy:tt) => {
        $crate::unexpected!($($rest)*)
    };

    // Match a comma with no parallel system type provided. This will output an error.
    (@par_system $processed:ty; (, $($rest:tt)*) ($comma:tt $($copy:tt)*)) => (
        $crate::unexpected!($comma)
    );

    // Match any other unexpected input. This will output an error.
    (@par_system $processed:ty; ($($rest:tt)*) $copy:tt) => (
        $crate::unexpected!($($rest)*)
    );

    ///////////////////////////////////////////////////////////////////////////////////////////////
    // Flush task macro.
    //
    // These patterns match the remainder of a flush task entry, which is only either a comma or
    // nothing.
    //
    // Note that we require two copies of the input token tree here. The first is matched on, and
    // the second is used to trigger errors.
    ///////////////////////////////////////////////////////////////////////////////////////////////

    // Match a comma.
    (@flush $processed:ty; (, $($rest:tt)*) $copy:tt) => (
        $crate::stages_internal!(@task ($crate::system::schedule::stage::Stage<$crate::system::Null, $crate::system::Null>, $processed); ($($rest)*) ($($rest)*))
    );

    // Match a colon. This is invalid and will output an error.
    (@flush $processed:ty; (: $($rest:tt)*) ($colon:tt $($copy:tt)*)) => (
        $crate::unexpected!($colon)
    );

    // Match nothing. This will only occur at the end of the input token tree, which is the only
    // time when a trailing comma is not required.
    (@flush $processed:ty; () ()) => (
        $crate::stages_internal!(@task ($crate::system::schedule::stage::Stage<$crate::system::Null, $crate::system::Null>, $processed); () ())
    );

    // Match any other unexpected input. This will output an error.
    (@flush $processed:ty; ($($rest:tt)*) $copy:tt) => (
        $crate::unexpected!($($rest)*)
    );
}
