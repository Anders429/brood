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

mod sealed;

use crate::{
    doc,
    hlist::define_null,
    registry::{
        ContainsParQuery,
        ContainsQuery,
        Registry,
    },
    system::{
        schedule::task::Task,
        ParSystem,
        System,
    },
};
use sealed::Sealed;

/// A single step in a stage.
///
/// A step is a single task in a stage, along with information about whether this task is the
/// beginning of a new stage or if it is a continuation of the current stage.
pub enum Stage<S, P> {
    /// The start of a new stage.
    Start(Task<S, P>),
    /// A continuation of the current stage.
    Continue(Task<S, P>),
    /// A manually-requested end to a stage.
    Flush,
}

/// A heterogeneous list of [`Stage`]s.
///
/// The ordered `Stage`s provided here define the actual stages of the schedule. Note that the
/// stages are defined inside-out, with the last of the heterogeneous list being the beginning of
/// the list of stages.
pub trait Stages<R, SFI, SVI, PFI, PVI, SP, SI, SQ, PP, PI, PQ>:
    Sealed<R, SFI, SVI, PFI, PVI, SP, SI, SQ, PP, PI, PQ>
where
    R: Registry,
{
}

define_null!();

impl<'a, R> Stages<R, Null, Null, Null, Null, Null, Null, Null, Null, Null, Null> for Null where
    R: Registry
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
        SQ,
        SQS,
        PP,
        PPS,
        PI,
        PIS,
        PQ,
        PQS,
    >
    Stages<
        R,
        (SFI, SFIS),
        (SVI, SVIS),
        (PFI, PFIS),
        (PVI, PVIS),
        (SP, SPS),
        (SI, SIS),
        (SQ, SQS),
        (PP, PPS),
        (PI, PIS),
        (PQ, PQS),
    > for (Stage<S, P>, L)
where
    R: ContainsQuery<'a, S::Filter, SFI, S::Views<'a>, SVI, SP, SI, SQ>
        + ContainsParQuery<'a, P::Filter, PFI, P::Views<'a>, PVI, PP, PI, PQ>,
    S: System + Send,
    P: ParSystem + Send,
    L: Stages<R, SFIS, SVIS, PFIS, PVIS, SPS, SIS, SQS, PPS, PIS, PQS>,
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
    /// use brood::{query::{filter, filter::Filter, result, views}, registry::{ContainsParQuery, ContainsQuery}, system::{schedule::stages, System, ParSystem}};
    ///
    /// // Define components.
    /// struct A;
    /// struct B;
    /// struct C;
    ///
    /// // Define a System.
    /// struct Foo;
    /// impl System for Foo {
    ///     type Filter = filter::None;
    ///     type Views<'a> = views!(&'a mut A, &'a B);
    ///
    ///     fn run<'a, R, FI, VI, P, I, Q>(&mut self, query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>)
    ///     where
    ///         R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
    ///     {
    ///         // Operate on result here.
    ///     }
    /// }
    ///
    /// // Define a Parallel System.
    /// struct Bar;
    /// impl ParSystem for Bar {
    ///     type Filter = filter::None;
    ///     type Views<'a> = views!(&'a B, &'a mut C);
    ///
    ///     fn run<'a, R, FI, VI, P, I, Q>(&mut self, query_results: result::ParIter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>)
    ///     where
    ///         R: ContainsParQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
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
        $crate::stages_internal!(@task ($crate::system::schedule::stage::Stage<$system, $crate::system::Null>, $processed); () ())
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

#[cfg(test)]
mod tests {
    use super::stages;
    use crate::{
        query::{
            filter,
            result,
            views,
        },
        registry,
        registry::ContainsQuery,
        system::System,
    };

    #[test]
    fn no_trailing_comma() {
        #[derive(Clone)]
        struct A(f32);
        #[derive(Clone)]
        struct B(f32);
        #[derive(Clone)]
        struct C(f32);
        #[derive(Clone)]
        struct D(f32);
        #[derive(Clone)]
        struct E(f32);

        type Registry = registry!(A, B, C, D, E);

        struct AB;

        impl System for AB {
            type Views<'a> = views!(&'a mut A, &'a mut B);
            type Filter = filter::None;

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            ) where
                R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                for result!(a, b) in query_results {
                    core::mem::swap(&mut a.0, &mut b.0);
                }
            }
        }

        struct CD;

        impl System for CD {
            type Views<'a> = views!(&'a mut C, &'a mut D);
            type Filter = filter::None;

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            ) where
                R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                for result!(c, d) in query_results {
                    core::mem::swap(&mut c.0, &mut d.0);
                }
            }
        }

        struct CE;

        impl System for CE {
            type Views<'a> = views!(&'a mut C, &'a mut E);
            type Filter = filter::None;

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            ) where
                R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                for result!(c, e) in query_results {
                    core::mem::swap(&mut c.0, &mut e.0);
                }
            }
        }

        // Lack of trailing comma here should not fail.
        type Stages = stages!(system: AB, system: CD, system: CE);
    }
}
