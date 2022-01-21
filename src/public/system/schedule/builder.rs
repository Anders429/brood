use crate::{
    internal::{
        query::claim::Claim,
        system,
        system::schedule::task::Task,
    },
    system::{
        schedule::{
            stage,
            stage::{Stage, Stages},
            Schedule,
        },
        ParSystem, System,
    },
};
use core::any::TypeId;
use hashbrown::HashSet;

pub enum RawTask<S, P> {
    Task(Task<S, P>),
    Flush,
}

pub struct Null;

pub trait RawTasks<'a> {
    type Stages: Stages<'a>;

    fn into_stages(
        self,
        mutable_claims: &mut HashSet<TypeId>,
        immutable_claims: &mut HashSet<TypeId>,
        mutable_buffer: &mut HashSet<TypeId>,
        immutable_buffer: &mut HashSet<TypeId>,
    ) -> Self::Stages;
}

impl<'a> RawTasks<'a> for Null {
    type Stages = stage::Null;

    fn into_stages(
        self,
        _mutable_claims: &mut HashSet<TypeId>,
        _immutable_claims: &mut HashSet<TypeId>,
        _mutable_buffer: &mut HashSet<TypeId>,
        _immutable_buffer: &mut HashSet<TypeId>,
    ) -> Self::Stages {
        stage::Null
    }
}

impl<'a, S, P, T> RawTasks<'a> for (RawTask<S, P>, T)
where
    S: System<'a> + Send,
    P: ParSystem<'a> + Send,
    T: RawTasks<'a>,
{
    type Stages = (Stage<S, P>, T::Stages);

    fn into_stages(
        self,
        mutable_claims: &mut HashSet<TypeId>,
        immutable_claims: &mut HashSet<TypeId>,
        mutable_buffer: &mut HashSet<TypeId>,
        immutable_buffer: &mut HashSet<TypeId>,
    ) -> Self::Stages {
        let prev_stages = self.1.into_stages(
            mutable_claims,
            immutable_claims,
            mutable_buffer,
            immutable_buffer,
        );

        match self.0 {
            RawTask::Task(task) => {
                mutable_buffer.clear();
                immutable_buffer.clear();

                // Identify this stage's claims on components.
                S::Views::claim(mutable_buffer, immutable_buffer);
                P::Views::claim(mutable_buffer, immutable_buffer);

                // Helper function to check whether the intersection betwen two sets is nonempty.
                fn intersects(a: &HashSet<TypeId>, b: &HashSet<TypeId>) -> bool {
                    a.intersection(b).next().is_some()
                }

                // If the claims are incompatible, a new stage must begin.
                //
                // Claims are incompatible if an immutable claim is made on a component already
                // mutable claimed, or if a mutable claim is made on a component already claimed at
                // all.
                if intersects(immutable_buffer, mutable_claims)
                    || intersects(mutable_buffer, mutable_claims)
                    || intersects(mutable_buffer, immutable_claims)
                {
                    (Stage::Start(task), prev_stages)
                } else {
                    (Stage::Continue(task), prev_stages)
                }
            }
            RawTask::Flush => {
                mutable_claims.clear();
                immutable_claims.clear();
                (Stage::Flush, prev_stages)
            }
        }
    }
}

pub struct Builder<T> {
    raw_tasks: T,
}

impl Builder<Null> {
    pub(super) fn new() -> Self {
        Self { raw_tasks: Null }
    }
}

impl<'a, T> Builder<T>
where
    T: RawTasks<'a>,
{
    pub fn system<S>(self, system: S) -> Builder<(RawTask<S, system::Null>, T)>
    where
        S: System<'a>,
    {
        Builder::<(RawTask<S, system::Null>, T)> {
            raw_tasks: (RawTask::Task(Task::Seq(system)), self.raw_tasks),
        }
    }

    pub fn par_system<S>(self, par_system: S) -> Builder<(RawTask<system::Null, S>, T)>
    where
        S: ParSystem<'a>,
    {
        Builder::<(RawTask<system::Null, S>, T)> {
            raw_tasks: (RawTask::Task(Task::Par(par_system)), self.raw_tasks),
        }
    }

    pub fn flush(self) -> Builder<(RawTask<system::Null, system::Null>, T)> {
        Builder::<(RawTask<system::Null, system::Null>, T)> {
            raw_tasks: (RawTask::Flush, self.raw_tasks),
        }
    }

    pub fn build(self) -> Schedule<T::Stages> {
        Schedule {
            stages: self.raw_tasks.into_stages(
                &mut HashSet::new(),
                &mut HashSet::new(),
                &mut HashSet::new(),
                &mut HashSet::new(),
            ),
        }
    }
}
