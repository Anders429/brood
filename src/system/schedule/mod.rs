//! A list of [`System`]s and [`ParSystem`]s to be run in stages.
//!
//! Stages are scheduled depending on the order in which the `System`s are provided. A [`Schedule`]
//! will proceed through its tasks in order and run as many of them as possible in parallel.
//! `System`s can run in parallel as long as their [`Views`] can be borrowed simulatenously.
//!
//! # Example
//! The below example will define a schedule that will execute both `SystemA` and `SystemB` in
//! parallel, since their views can be borrowed simultaneously.
//!
//! ``` rust
//! use brood::{
//!     query::{
//!         filter,
//!         filter::Filter,
//!         result,
//!         Views,
//!     },
//!     registry::ContainsQuery,
//!     system::{
//!         schedule,
//!         schedule::task,
//!         System,
//!     },
//! };
//!
//! // Define components.
//! struct Foo(usize);
//! struct Bar(bool);
//! struct Baz(f64);
//!
//! struct SystemA;
//!
//! impl System for SystemA {
//!     type Views<'a> = Views!(&'a mut Foo, &'a Bar);
//!     type Filter = filter::None;
//!
//!     fn run<'a, R, FI, VI, P, I, Q>(
//!         &mut self,
//!         query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
//!     ) where
//!         R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
//!     {
//!         for result!(foo, bar) in query_results {
//!             // Do something...
//!         }
//!     }
//! }
//!
//! struct SystemB;
//!
//! impl System for SystemB {
//!     type Views<'a> = Views!(&'a mut Baz, &'a Bar);
//!     type Filter = filter::None;
//!
//!     fn run<'a, R, FI, VI, P, I, Q>(
//!         &mut self,
//!         query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
//!     ) where
//!         R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
//!     {
//!         for result!(baz, bar) in query_results {
//!             // Do something...
//!         }
//!     }
//! }
//!
//! let schedule = schedule!(task::System(SystemA), task::System(SystemB));
//! ```
//!
//! [`ParSystem`]: crate::system::ParSystem
//! [`Schedule`]: trait@crate::system::schedule::Schedule
//! [`System`]: crate::system::System
//! [`Views`]: trait@crate::query::view::Views

pub mod task;

mod claim;
mod scheduler;
mod sealed;
mod sendable;
mod stage;
mod stager;
mod stages;

pub(crate) use stages::Stages;

use crate::{
    doc,
    registry::Registry,
};
use scheduler::Scheduler;
use sealed::Sealed;
use stage::Stage;
use stager::Stager;
use task::Task;

/// A list of tasks, scheduled to run in stages.
///
/// This is a heterogeneous list of [`System`]s and [`ParSystem`]s. Stages are created at compile
/// time based on the [`Views`] of each system, ensuring the borrows will follow Rust's borrowing
/// rules.
///
/// The easiest way to create a `Schedule` is by using the [`schedule!`] macro.
///
/// [`ParSystem`]: crate::system::ParSystem
/// [`schedule!`]: crate::system::schedule!
/// [`System`]: crate::system::System
/// [`Views`]: trait@crate::query::view::Views
pub trait Schedule<'a, R, I, P, RI, SFI, SVI, SP, SI, SQ>:
    Sealed<'a, R, I, P, RI, SFI, SVI, SP, SI, SQ>
where
    R: Registry,
{
}

impl<'a, R, T, I, P, RI, SFI, SVI, SP, SI, SQ> Schedule<'a, R, I, P, RI, SFI, SVI, SP, SI, SQ> for T
where
    R: Registry,
    T: Sealed<'a, R, I, P, RI, SFI, SVI, SP, SI, SQ>,
{
}

doc::non_root_macro! {
    /// Macro for defining a heterogeneous list of tasks.
    ///
    /// Note that this is a list of tasks, not systems. Specifically, this is a list of
    /// [`task::System`]s and [`task::ParSystem`]s.
    ///
    /// # Example
    /// ``` rust
    /// use brood::{
    ///     query::{
    ///         filter,
    ///         filter::Filter,
    ///         result,
    ///         Views,
    ///     },
    ///     registry::{
    ///         ContainsParQuery,
    ///         ContainsQuery,
    ///     },
    ///     system::{
    ///         schedule,
    ///         schedule::task,
    ///         ParSystem,
    ///         System,
    ///     },
    /// };
    ///
    /// // Define components.
    /// struct Foo(usize);
    /// struct Bar(bool);
    /// struct Baz(f64);
    ///
    /// struct SystemA;
    ///
    /// impl System for SystemA {
    ///     type Views<'a> = Views!(&'a mut Foo, &'a Bar);
    ///     type Filter = filter::None;
    ///
    ///     fn run<'a, R, FI, VI, P, I, Q>(
    ///         &mut self,
    ///         query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
    ///     ) where
    ///         R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
    ///     {
    ///         // Do something..
    ///     }
    /// }
    ///
    /// struct SystemB;
    ///
    /// impl ParSystem for SystemB {
    ///     type Views<'a> = Views!(&'a mut Baz, &'a Bar);
    ///     type Filter = filter::None;
    ///
    ///     fn run<'a, R, FI, VI, P, I, Q>(
    ///         &mut self,
    ///         query_results: result::ParIter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
    ///     ) where
    ///         R: ContainsParQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
    ///     {
    ///         // Do something..
    ///     }
    /// }
    ///
    /// let schedule = schedule!(task::System(SystemA), task::System(SystemB));
    /// ```
    ///
    /// [`task::ParSystem`]: crate::system::schedule::task::ParSystem
    /// [`task::System`]: crate::system::schedule::task::System
    macro_rules! schedule {
        ($task:expr $(,$tasks:expr)* $(,)?) => (
            ($task, $crate::system::schedule::schedule!($($tasks,)*))
        );
        () => (
            $crate::system::schedule::task::Null
        );
    }
}

/// Nesting this macro definition in a module is necessary to unambiguate the import of the macro.
pub(crate) mod inner {
    use crate::doc;

    doc::non_root_macro! {
        /// Macro for defining the type of a schedule.
        ///
        /// This macro is used to define the type of a schedule made up of a list of tasks.
        ///
        /// # Example
        /// ``` rust
        /// use brood::{
        ///     query::{
        ///         filter,
        ///         filter::Filter,
        ///         result,
        ///         Views,
        ///     },
        ///     registry::{
        ///         ContainsParQuery,
        ///         ContainsQuery,
        ///     },
        ///     system::{
        ///         schedule::task,
        ///         ParSystem,
        ///         Schedule,
        ///         System,
        ///     },
        /// };
        ///
        /// // Define components.
        /// struct Foo(usize);
        /// struct Bar(bool);
        /// struct Baz(f64);
        ///
        /// struct SystemA;
        ///
        /// impl System for SystemA {
        ///     type Views<'a> = Views!(&'a mut Foo, &'a Bar);
        ///     type Filter = filter::None;
        ///
        ///     fn run<'a, R, FI, VI, P, I, Q>(
        ///         &mut self,
        ///         query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
        ///     ) where
        ///         R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
        ///     {
        ///         // Do something..
        ///     }
        /// }
        ///
        /// struct SystemB;
        ///
        /// impl ParSystem for SystemB {
        ///     type Views<'a> = Views!(&'a mut Baz, &'a Bar);
        ///     type Filter = filter::None;
        ///
        ///     fn run<'a, R, FI, VI, P, I, Q>(
        ///         &mut self,
        ///         query_results: result::ParIter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
        ///     ) where
        ///         R: ContainsParQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
        ///     {
        ///         // Do something..
        ///     }
        /// }
        ///
        /// type Schedule = Schedule!(task::System<SystemA>, task::System<SystemB>);
        /// ```
        macro_rules! Schedule {
            ($task:ty $(,$tasks:ty)* $(,)?) => (
                ($task, $crate::system::schedule::Schedule!($($tasks,)*))
            );
            () => (
                $crate::system::schedule::task::Null
            );
        }
    }
}

pub use inner::Schedule;

#[cfg(test)]
mod tests {
    use super::Sealed as Schedule;
    use crate::{
        entity,
        query::{
            filter,
            result,
            Views,
        },
        registry::{
            ContainsParQuery,
            ContainsQuery,
        },
        system::{
            schedule::{
                stage,
                stages,
                task,
            },
            ParSystem,
            System,
        },
        Registry,
    };
    use core::any::TypeId;

    struct A;
    struct B;
    struct C;
    struct D;
    struct E;

    type Registry = Registry!(A, B, C, D, E);

    #[test]
    fn null() {
        assert_eq!(
            TypeId::of::<<task::Null as Schedule<'_, Registry, _, _, _, _, _, _, _, _>>::Stages>(),
            TypeId::of::<stages::Null>()
        );
    }

    #[test]
    fn single_system_immut_a() {
        struct ImmutA;

        impl System for ImmutA {
            type Views<'a> = Views!(&'a A);
            type Filter = filter::None;

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                _query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            ) where
                R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                unreachable!()
            }
        }

        assert_eq!(
            TypeId::of::<
                <(task::System<ImmutA>, task::Null) as Schedule<
                    '_,
                    Registry,
                    _,
                    _,
                    _,
                    _,
                    _,
                    _,
                    _,
                    _,
                >>::Stages,
            >(),
            TypeId::of::<((&mut task::System<ImmutA>, stage::Null), stages::Null)>()
        );
    }

    #[test]
    fn single_system_mut_a() {
        struct MutA;

        impl System for MutA {
            type Views<'a> = Views!(&'a mut A);
            type Filter = filter::None;

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                _query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            ) where
                R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                unreachable!()
            }
        }

        assert_eq!(
            TypeId::of::<
                <(task::System<MutA>, task::Null) as Schedule<
                    '_,
                    Registry,
                    _,
                    _,
                    _,
                    _,
                    _,
                    _,
                    _,
                    _,
                >>::Stages,
            >(),
            TypeId::of::<((&mut task::System<MutA>, stage::Null), stages::Null)>()
        );
    }

    #[test]
    fn single_system_option_immut_a() {
        struct OptionImmutA;

        impl System for OptionImmutA {
            type Views<'a> = Views!(Option<&'a A>);
            type Filter = filter::None;

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                _query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            ) where
                R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                unreachable!()
            }
        }

        assert_eq!(
            TypeId::of::<
                <(task::System<OptionImmutA>, task::Null) as Schedule<
                    '_,
                    Registry,
                    _,
                    _,
                    _,
                    _,
                    _,
                    _,
                    _,
                    _,
                >>::Stages,
            >(),
            TypeId::of::<((&mut task::System<OptionImmutA>, stage::Null), stages::Null)>()
        );
    }

    #[test]
    fn single_system_option_mut_a() {
        struct OptionMutA;

        impl System for OptionMutA {
            type Views<'a> = Views!(Option<&'a mut A>);
            type Filter = filter::None;

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                _query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            ) where
                R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                unreachable!()
            }
        }

        assert_eq!(
            TypeId::of::<
                <(task::System<OptionMutA>, task::Null) as Schedule<
                    '_,
                    Registry,
                    _,
                    _,
                    _,
                    _,
                    _,
                    _,
                    _,
                    _,
                >>::Stages,
            >(),
            TypeId::of::<((&mut task::System<OptionMutA>, stage::Null), stages::Null)>()
        );
    }

    #[test]
    fn single_system_entity_identifier() {
        struct EntityIdentifier;

        impl System for EntityIdentifier {
            type Views<'a> = Views!(entity::Identifier);
            type Filter = filter::None;

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                _query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            ) where
                R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                unreachable!()
            }
        }

        assert_eq!(
            TypeId::of::<
                <(task::System<EntityIdentifier>, task::Null) as Schedule<
                    '_,
                    Registry,
                    _,
                    _,
                    _,
                    _,
                    _,
                    _,
                    _,
                    _,
                >>::Stages,
            >(),
            TypeId::of::<(
                (&mut task::System<EntityIdentifier>, stage::Null),
                stages::Null
            )>()
        );
    }

    #[test]
    fn single_par_system_immut_a() {
        struct ImmutA;

        impl ParSystem for ImmutA {
            type Views<'a> = Views!(&'a A);
            type Filter = filter::None;

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                _query_results: result::ParIter<
                    'a,
                    R,
                    Self::Filter,
                    FI,
                    Self::Views<'a>,
                    VI,
                    P,
                    I,
                    Q,
                >,
            ) where
                R: ContainsParQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                unreachable!()
            }
        }

        assert_eq!(
            TypeId::of::<
                <(task::ParSystem<ImmutA>, task::Null) as Schedule<
                    '_,
                    Registry,
                    _,
                    _,
                    _,
                    _,
                    _,
                    _,
                    _,
                    _,
                >>::Stages,
            >(),
            TypeId::of::<((&mut task::ParSystem<ImmutA>, stage::Null), stages::Null)>()
        );
    }

    #[test]
    fn single_par_system_mut_a() {
        struct MutA;

        impl ParSystem for MutA {
            type Views<'a> = Views!(&'a mut A);
            type Filter = filter::None;

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                _query_results: result::ParIter<
                    'a,
                    R,
                    Self::Filter,
                    FI,
                    Self::Views<'a>,
                    VI,
                    P,
                    I,
                    Q,
                >,
            ) where
                R: ContainsParQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                unreachable!()
            }
        }

        assert_eq!(
            TypeId::of::<
                <(task::ParSystem<MutA>, task::Null) as Schedule<
                    '_,
                    Registry,
                    _,
                    _,
                    _,
                    _,
                    _,
                    _,
                    _,
                    _,
                >>::Stages,
            >(),
            TypeId::of::<((&mut task::ParSystem<MutA>, stage::Null), stages::Null)>()
        );
    }

    #[test]
    fn single_par_system_option_immut_a() {
        struct OptionImmutA;

        impl ParSystem for OptionImmutA {
            type Views<'a> = Views!(Option<&'a A>);
            type Filter = filter::None;

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                _query_results: result::ParIter<
                    'a,
                    R,
                    Self::Filter,
                    FI,
                    Self::Views<'a>,
                    VI,
                    P,
                    I,
                    Q,
                >,
            ) where
                R: ContainsParQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                unreachable!()
            }
        }

        assert_eq!(
            TypeId::of::<
                <(task::ParSystem<OptionImmutA>, task::Null) as Schedule<
                    '_,
                    Registry,
                    _,
                    _,
                    _,
                    _,
                    _,
                    _,
                    _,
                    _,
                >>::Stages,
            >(),
            TypeId::of::<(
                (&mut task::ParSystem<OptionImmutA>, stage::Null),
                stages::Null
            )>()
        );
    }

    #[test]
    fn single_par_system_option_mut_a() {
        struct OptionMutA;

        impl ParSystem for OptionMutA {
            type Views<'a> = Views!(Option<&'a mut A>);
            type Filter = filter::None;

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                _query_results: result::ParIter<
                    'a,
                    R,
                    Self::Filter,
                    FI,
                    Self::Views<'a>,
                    VI,
                    P,
                    I,
                    Q,
                >,
            ) where
                R: ContainsParQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                unreachable!()
            }
        }

        assert_eq!(
            TypeId::of::<
                <(task::ParSystem<OptionMutA>, task::Null) as Schedule<
                    '_,
                    Registry,
                    _,
                    _,
                    _,
                    _,
                    _,
                    _,
                    _,
                    _,
                >>::Stages,
            >(),
            TypeId::of::<(
                (&mut task::ParSystem<OptionMutA>, stage::Null),
                stages::Null
            )>()
        );
    }

    #[test]
    fn single_par_system_entity_identifier() {
        struct EntityIdentifier;

        impl ParSystem for EntityIdentifier {
            type Views<'a> = Views!(entity::Identifier);
            type Filter = filter::None;

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                _query_results: result::ParIter<
                    'a,
                    R,
                    Self::Filter,
                    FI,
                    Self::Views<'a>,
                    VI,
                    P,
                    I,
                    Q,
                >,
            ) where
                R: ContainsParQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                unreachable!()
            }
        }

        assert_eq!(
            TypeId::of::<
                <(task::ParSystem<EntityIdentifier>, task::Null) as Schedule<
                    '_,
                    Registry,
                    _,
                    _,
                    _,
                    _,
                    _,
                    _,
                    _,
                    _,
                >>::Stages,
            >(),
            TypeId::of::<(
                (&mut task::ParSystem<EntityIdentifier>, stage::Null),
                stages::Null
            )>()
        );
    }

    #[test]
    fn multiple_stages() {
        struct AB;

        impl System for AB {
            type Views<'a> = Views!(&'a mut A, &'a mut B);
            type Filter = filter::None;

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                _query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            ) where
                R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                unimplemented!()
            }
        }

        struct CD;

        impl System for CD {
            type Views<'a> = Views!(&'a mut C, &'a mut D);
            type Filter = filter::None;

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                _query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            ) where
                R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                unimplemented!()
            }
        }

        struct CE;

        impl System for CE {
            type Views<'a> = Views!(&'a mut C, &'a mut E);
            type Filter = filter::None;

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                _query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            ) where
                R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                unimplemented!()
            }
        }

        assert_eq!(
            TypeId::of::<
                <(
                    task::System<AB>,
                    (task::System<CD>, (task::System<CE>, task::Null))
                ) as Schedule<'_, Registry, _, _, _, _, _, _, _, _>>::Stages,
            >(),
            TypeId::of::<(
                (&mut task::System<AB>, (&mut task::System<CD>, stage::Null)),
                ((&mut task::System<CE>, stage::Null), stages::Null)
            )>()
        );
    }
}
