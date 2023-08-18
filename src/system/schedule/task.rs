use crate::{
    hlist::Null,
    query::{
        view::{
            self,
            Claims,
        },
        Query,
    },
    registry,
    resource,
    system::{
        schedule::{
            stage::SendPtr,
            Stage,
        },
        ParSystem,
        System,
    },
    world::World,
};

pub enum Task<System, ParSystem> {
    System(System),
    ParSystem(ParSystem),
}

impl<System, ParSystem> Task<System, ParSystem> {
    pub(crate) fn run<
        'a,
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
    >(
        &mut self,
        world: SendPtr<World<Registry, Resources>>,
    ) where
        Registry: registry::ContainsQuery<'a, System::Filter, System::Views<'a>, QueryIndices>
            + registry::ContainsViews<'a, System::EntryViews<'a>, EntryIndices>
            + registry::ContainsParQuery<'a, ParSystem::Filter, ParSystem::Views<'a>, ParQueryIndices>
            + registry::ContainsViews<'a, ParSystem::EntryViews<'a>, ParEntryIndices>
            + 'a,
        Resources: resource::ContainsViews<'a, System::ResourceViews<'a>, ResourceViewIndices>
            + resource::ContainsViews<'a, ParSystem::ResourceViews<'a>, ParResourceViewIndices>
            + 'a,
        System: self::System,
        System::Views<'a>: Send,
        System::ResourceViews<'a>: Send,
        System::EntryViews<'a>: view::Disjoint<System::Views<'a>, Registry, DisjointIndices> + Send,
        ParSystem: self::ParSystem,
        ParSystem::Views<'a>: Send,
        ParSystem::ResourceViews<'a>: Send,
        ParSystem::EntryViews<'a>:
            view::Disjoint<ParSystem::Views<'a>, Registry, ParDisjointIndices> + Send,
    {
        match self {
            Self::System(system) => {
                let result = unsafe {
                    (*world.0).query(Query::<
                        System::Views<'a>,
                        System::Filter,
                        System::ResourceViews<'a>,
                        System::EntryViews<'a>,
                    >::new())
                };
                system.run(result);
            }
            Self::ParSystem(par_system) => {
                let result = unsafe {
                    (*world.0).par_query(Query::<
                        ParSystem::Views<'a>,
                        ParSystem::Filter,
                        ParSystem::ResourceViews<'a>,
                        ParSystem::EntryViews<'a>,
                    >::new())
                };
                par_system.run(result);
            }
        }
    }

    fn claims<
        'a,
        Registry,
        Resources,
        ViewsIndices,
        EntryViewsIndices,
        ResourceViewsIndices,
        ParViewsIndices,
        ParEntryViewsIndices,
        ParResourceViewsIndices,
    >(
        &self,
    ) -> (Registry::Claims, Resources::Claims)
    where
        System: self::System,
        ParSystem: self::ParSystem,
        Registry: registry::ContainsViews<'a, System::Views<'a>, ViewsIndices>
            + registry::ContainsViews<'a, System::EntryViews<'a>, EntryViewsIndices>
            + registry::ContainsViews<'a, ParSystem::Views<'a>, ParViewsIndices>
            + registry::ContainsViews<'a, ParSystem::EntryViews<'a>, ParEntryViewsIndices>,
        Resources: resource::ContainsViews<'a, System::ResourceViews<'a>, ResourceViewsIndices>
            + resource::ContainsViews<'a, ParSystem::ResourceViews<'a>, ParResourceViewsIndices>,
    {
        match self {
            Self::System(_) => (
                unsafe {
                    <Registry as registry::contains::views::Sealed<
                        'a,
                        System::Views<'a>,
                        ViewsIndices,
                    >>::claims()
                    .merge_unchecked(&<Registry as registry::contains::views::Sealed<
                        'a,
                        System::EntryViews<'a>,
                        EntryViewsIndices,
                    >>::claims())
                },
                <Resources as resource::contains::views::Sealed<
                    'a,
                    System::ResourceViews<'a>,
                    ResourceViewsIndices,
                >>::claims(),
            ),
            Self::ParSystem(_) => (
                unsafe {
                    <Registry as registry::contains::views::Sealed<
                        'a,
                        ParSystem::Views<'a>,
                        ParViewsIndices,
                    >>::claims()
                    .merge_unchecked(&<Registry as registry::contains::views::Sealed<
                        'a,
                        ParSystem::EntryViews<'a>,
                        ParEntryViewsIndices,
                    >>::claims())
                },
                <Resources as resource::contains::views::Sealed<
                    'a,
                    ParSystem::ResourceViews<'a>,
                    ParResourceViewsIndices,
                >>::claims(),
            ),
        }
    }
}

/// A heterogeneous list of tasks.
///
/// A task is a system to be run on a [`World`]. A set of tasks implementing this trait can be
/// converted into a set of stages, dividing the tasks into parallelizable groups.
pub trait Tasks<'a, Registry, Resources, Indices>:
    Sealed<'a, Registry, Resources, Indices>
{
}

impl<'a, Tasks, Registry, Resources, Indices> self::Tasks<'a, Registry, Resources, Indices>
    for Tasks
where
    Tasks: Sealed<'a, Registry, Resources, Indices>,
{
}

pub trait Sealed<'a, Registry, Resources, Indices> {
    type Stages;

    fn into_stages(self) -> Self::Stages
    where
        Registry: registry::Claims,
        Resources: resource::Claims;
    fn into_stages_internal(self) -> (Registry::Claims, Resources::Claims, Self::Stages)
    where
        Registry: registry::Claims,
        Resources: resource::Claims;
}

impl<Registry, Resources> Sealed<'_, Registry, Resources, Null> for Null {
    type Stages = Null;

    fn into_stages(self) -> Self::Stages
    where
        Registry: registry::Claims,
        Resources: resource::Claims,
    {
        Null
    }

    fn into_stages_internal(self) -> (Registry::Claims, Resources::Claims, Self::Stages)
    where
        Registry: registry::Claims,
        Resources: resource::Claims,
    {
        (Registry::empty_claims(), Resources::empty_claims(), Null)
    }
}

impl<
        'a,
        Tasks,
        System,
        ParSystem,
        Registry,
        Resources,
        ViewsIndices,
        EntryViewsIndices,
        ResourceViewsIndices,
        ParViewsIndices,
        ParEntryViewsIndices,
        ParResourceViewsIndices,
        Indices,
    >
    Sealed<
        'a,
        Registry,
        Resources,
        (
            ViewsIndices,
            EntryViewsIndices,
            ResourceViewsIndices,
            ParViewsIndices,
            ParEntryViewsIndices,
            ParResourceViewsIndices,
            Indices,
        ),
    > for (Task<System, ParSystem>, Tasks)
where
    Tasks: Sealed<'a, Registry, Resources, Indices>,
    System: self::System,
    ParSystem: self::ParSystem,
    Registry: registry::ContainsViews<'a, System::Views<'a>, ViewsIndices>
        + registry::ContainsViews<'a, System::EntryViews<'a>, EntryViewsIndices>
        + registry::ContainsViews<'a, ParSystem::Views<'a>, ParViewsIndices>
        + registry::ContainsViews<'a, ParSystem::EntryViews<'a>, ParEntryViewsIndices>,
    Resources: resource::ContainsViews<'a, System::ResourceViews<'a>, ResourceViewsIndices>
        + resource::ContainsViews<'a, ParSystem::ResourceViews<'a>, ParResourceViewsIndices>,
{
    type Stages = (Stage<System, ParSystem>, Tasks::Stages);

    fn into_stages(self) -> Self::Stages
    where
        Registry: registry::Claims,
        Resources: resource::Claims,
    {
        self.into_stages_internal().2
    }

    fn into_stages_internal(self) -> (Registry::Claims, Resources::Claims, Self::Stages)
    where
        Registry: registry::Claims,
        Resources: resource::Claims,
    {
        let (component_claims, resource_claims, stages) = self.1.into_stages_internal();
        let (new_component_claims, new_resource_claims) = self.0.claims::<Registry, Resources, ViewsIndices, EntryViewsIndices, ResourceViewsIndices, ParViewsIndices, ParEntryViewsIndices, ParResourceViewsIndices>();
        match (
            component_claims.try_merge(&new_component_claims),
            resource_claims.try_merge(&new_resource_claims),
        ) {
            (Some(component_claims), Some(resource_claims)) => (
                component_claims,
                resource_claims,
                (Stage::Continue(self.0), stages),
            ),
            _ => (
                Registry::empty_claims(),
                Resources::empty_claims(),
                (Stage::Start(self.0), stages),
            ),
        }
    }
}
