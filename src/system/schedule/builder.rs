use crate::{
    system,
    system::{
        schedule::{
            raw_task,
            raw_task::{
                RawTask,
                RawTasks,
            },
            task::Task,
            Schedule,
        },
        ParSystem,
        System,
    },
};
use fnv::FnvBuildHasher;
use hashbrown::HashSet;

/// A [`Schedule`] builder.
///
/// A `Builder` can be used to construct a `Schedule` of [`System`]s or [`ParSystem`]s to be run in
/// order. Systems that can be executed in parallel are combined into a single [`Stage`].
///
/// All post-processing of the [`World`] is deferred until after the full schedule is run. However,
/// a [`flush`] can be requested between `System`s, which ends the current `Stage` and executes
/// post-processing for every `System` that has already executed.
///
/// # Example
/// The below example will execute both `SystemA` and `SystemB` in parallel, since their views can
/// be borrowed simultaneously.
///
/// ``` rust
/// use brood::{
///     query::{
///         filter,
///         filter::Filter,
///         result,
///         views,
///     },
///     registry::ContainsQuery,
///     system::{
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
/// impl<'a> System<'a> for SystemA {
///     type Views = views!(&'a mut Foo, &'a Bar);
///     type Filter = filter::None;
///
///     fn run<R, FI, VI, P, I, Q>(
///         &mut self,
///         query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views, VI, P, I, Q>,
///     ) where
///         R: ContainsQuery<'a, Self::Filter, FI, Self::Views, VI, P, I, Q> + 'a,
///     {
///         for result!(foo, bar) in query_results {
///             // Do something...
///         }
///     }
/// }
///
/// struct SystemB;
///
/// impl<'a> System<'a> for SystemB {
///     type Views = views!(&'a mut Baz, &'a Bar);
///     type Filter = filter::None;
///
///     fn run<R, FI, VI, P, I, Q>(
///         &mut self,
///         query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views, VI, P, I, Q>,
///     ) where
///         R: ContainsQuery<'a, Self::Filter, FI, Self::Views, VI, P, I, Q> + 'a,
///     {
///         for result!(baz, bar) in query_results {
///             // Do something...
///         }
///     }
/// }
///
/// let schedule = Schedule::builder().system(SystemA).system(SystemB).build();
/// ```
///
/// [`flush`]: crate::system::schedule::Builder::flush()
/// [`Schedule`]: crate::system::schedule::Schedule
/// [`Stage`]: crate::system::schedule::stage::Stage
/// [`World`]: crate::world::World
pub struct Builder<T> {
    raw_tasks: T,
}

impl Builder<raw_task::Null> {
    pub(super) fn new() -> Self {
        Self {
            raw_tasks: raw_task::Null,
        }
    }
}

impl<'a, T> Builder<T>
where
    T: RawTasks<'a>,
{
    /// Add a [`System`] to the [`Schedule`].
    ///
    /// [`Schedule`]: crate::system::schedule::Schedule
    /// [`System`]: crate::system::System
    pub fn system<S>(self, system: S) -> Builder<(RawTask<S, system::Null>, T)>
    where
        S: System<'a>,
        S::Views: Send,
    {
        Builder::<(RawTask<S, system::Null>, T)> {
            raw_tasks: (RawTask::Task(Task::Seq(system)), self.raw_tasks),
        }
    }

    /// Add a [`ParSystem`] to the [`Schedule`].
    ///
    /// [`ParSystem`]: crate::system::ParSystem
    /// [`Schedule`]: crate::system::schedule::Schedule
    pub fn par_system<S>(self, par_system: S) -> Builder<(RawTask<system::Null, S>, T)>
    where
        S: ParSystem<'a>,
    {
        Builder::<(RawTask<system::Null, S>, T)> {
            raw_tasks: (RawTask::Task(Task::Par(par_system)), self.raw_tasks),
        }
    }

    /// Add a step to execute post-processing on all [`System`]s from previous stages that have not
    /// yet had post-processing executed on them.
    ///
    /// [`System`]: crate::system::System
    pub fn flush(self) -> Builder<(RawTask<system::Null, system::Null>, T)> {
        Builder::<(RawTask<system::Null, system::Null>, T)> {
            raw_tasks: (RawTask::Flush, self.raw_tasks),
        }
    }

    /// Create a [`Schedule`] containing all [`Stages`] added to the `Builder`, consuming the
    /// `Builder`.
    ///
    /// [`Schedule`]: crate::system::schedule::Schedule
    /// [`Stages`]: crate::system::schedule::stage::Stages
    pub fn build(self) -> Schedule<T::Stages> {
        Schedule {
            stages: self.raw_tasks.into_stages(
                &mut HashSet::with_hasher(FnvBuildHasher::default()),
                &mut HashSet::with_hasher(FnvBuildHasher::default()),
                &mut HashSet::with_hasher(FnvBuildHasher::default()),
                &mut HashSet::with_hasher(FnvBuildHasher::default()),
            ),
        }
    }
}
