use crate::{
    hlist::Null,
    query::view,
    registry,
    registry::Registry,
    resource,
    system::{
        schedule::Task,
        ParSystem,
        System,
    },
    world::World,
};

pub enum Stage<System, ParSystem> {
    Start(Task<System, ParSystem>),
    Continue(Task<System, ParSystem>),
}

/// A heterogeneous list of stages.
///
/// A list of stages is a list of tasks that has been annotated with stage boundary information. It
/// is created when building a schedule.
pub trait Stages<'a, Registry, Resources, Indices>:
    Sealed<'a, Registry, Resources, Indices>
{
}

impl<'a, Stages, Registry, Resources, Indices> self::Stages<'a, Registry, Resources, Indices>
    for Stages
where
    Stages: Sealed<'a, Registry, Resources, Indices>,
{
}

pub struct SendPtr<T>(pub(crate) *mut T);

unsafe impl<T> Send for SendPtr<T> {}

unsafe impl<T> Sync for SendPtr<T> {}

impl<T> Clone for SendPtr<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for SendPtr<T> {}

pub trait Sealed<'a, Registry, Resources, Indices>: Send {
    fn run(&mut self, world: SendPtr<World<Registry, Resources>>)
    where
        Registry: self::Registry;
    fn defer(&mut self, world: SendPtr<World<Registry, Resources>>)
    where
        Registry: self::Registry;
    fn run_current(&mut self, world: SendPtr<World<Registry, Resources>>)
    where
        Registry: self::Registry;
}

impl<Registry, Resources> Sealed<'_, Registry, Resources, Null> for Null {
    fn run(&mut self, _world: SendPtr<World<Registry, Resources>>)
    where
        Registry: self::Registry,
    {
    }
    fn defer(&mut self, _world: SendPtr<World<Registry, Resources>>)
    where
        Registry: self::Registry,
    {
    }
    fn run_current(&mut self, _world: SendPtr<World<Registry, Resources>>)
    where
        Registry: self::Registry,
    {
    }
}

impl<
        'a,
        Stages,
        System,
        ParSystem,
        Registry,
        Resources,
        QueryIndices,
        ResourceViewIndices,
        DisjointIndices,
        EntryIndices,
        ParQueryIndices,
        ParResourceViewIndices,
        ParDisjointIndices,
        ParEntryIndices,
        Indices,
    >
    Sealed<
        'a,
        Registry,
        Resources,
        (
            QueryIndices,
            ResourceViewIndices,
            DisjointIndices,
            EntryIndices,
            ParQueryIndices,
            ParResourceViewIndices,
            ParDisjointIndices,
            ParEntryIndices,
            Indices,
        ),
    > for (Stage<System, ParSystem>, Stages)
where
    Stages: Sealed<'a, Registry, Resources, Indices>,
    Registry: registry::ContainsQuery<'a, System::Filter, System::Views<'a>, QueryIndices>
        + registry::ContainsViews<'a, System::EntryViews<'a>, EntryIndices>
        + registry::ContainsParQuery<'a, ParSystem::Filter, ParSystem::Views<'a>, ParQueryIndices>
        + registry::ContainsViews<'a, ParSystem::EntryViews<'a>, ParEntryIndices>
        + 'a,
    Resources: resource::ContainsViews<'a, System::ResourceViews<'a>, ResourceViewIndices>
        + resource::ContainsViews<'a, ParSystem::ResourceViews<'a>, ParResourceViewIndices>
        + 'a,
    System: self::System + Send,
    System::Views<'a>: Send,
    System::ResourceViews<'a>: Send,
    System::EntryViews<'a>: view::Disjoint<System::Views<'a>, Registry, DisjointIndices> + Send,
    ParSystem: self::ParSystem + Send,
    ParSystem::Views<'a>: Send,
    ParSystem::ResourceViews<'a>: Send,
    ParSystem::EntryViews<'a>:
        view::Disjoint<ParSystem::Views<'a>, Registry, ParDisjointIndices> + Send,
{
    fn run(&mut self, world: SendPtr<World<Registry, Resources>>)
    where
        Registry: self::Registry,
    {
        self.defer(world);
        self.run_current(world);
    }

    fn defer(&mut self, world: SendPtr<World<Registry, Resources>>)
    where
        Registry: self::Registry,
    {
        match self.0 {
            Stage::Start(_) => {
                self.1.run(world);
            }
            Stage::Continue(_) => {
                self.1.defer(world);
            }
        }
    }

    fn run_current(&mut self, world: SendPtr<World<Registry, Resources>>)
    where
        Registry: self::Registry,
    {
        match &mut self.0 {
            Stage::Start(task) => {
                task.run(world);
            }
            Stage::Continue(task) => {
                rayon::join(|| self.1.run_current(world), || task.run(world));
            }
        }
    }
}
