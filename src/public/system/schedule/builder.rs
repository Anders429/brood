use crate::{
    internal::{
        system,
        system::schedule::{
            raw_task,
            raw_task::{RawTask, RawTasks},
        },
    },
    system::{schedule::{Schedule, task::Task}, ParSystem, System},
};
use hashbrown::HashSet;

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
