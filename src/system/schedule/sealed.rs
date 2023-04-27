use crate::{
    registry::Registry,
    system::schedule::{
        Scheduler,
        Stages,
    },
};

pub trait Sealed<'a, R, Resources, Indices>
where
    R: Registry,
{
    type QueryIndicesLists;
    type ResourceViewsIndicesLists;
    type DisjointIndicesLists;
    type EntryIndicesLists;
    type EntryViewsFilterIndicesLists;
    type Stages: Stages<
        'a,
        R,
        Resources,
        Self::QueryIndicesLists,
        Self::ResourceViewsIndicesLists,
        Self::DisjointIndicesLists,
        Self::EntryIndicesLists,
        Self::EntryViewsFilterIndicesLists,
    >;

    fn as_stages(&'a mut self) -> Self::Stages;
}

impl<
        'a,
        R,
        Resources,
        T,
        I,
        P,
        RI,
        MergeParametersList,
        ResourcesIndicesLists,
        ResourcesContainmentsLists,
        ResourcesInverseIndicesLists,
        QueryIndicesLists,
        ResourceViewsIndicesLists,
        DisjointIndicesLists,
        EntryIndicesLists,
        EntryViewsFilterIndicesLists,
    >
    Sealed<
        'a,
        R,
        Resources,
        (
            I,
            P,
            RI,
            MergeParametersList,
            ResourcesIndicesLists,
            ResourcesContainmentsLists,
            ResourcesInverseIndicesLists,
            QueryIndicesLists,
            ResourceViewsIndicesLists,
            DisjointIndicesLists,
            EntryIndicesLists,
            EntryViewsFilterIndicesLists,
        ),
    > for T
where
    R: Registry,
    T: Scheduler<
        'a,
        R,
        Resources,
        I,
        P,
        RI,
        MergeParametersList,
        ResourcesIndicesLists,
        ResourcesContainmentsLists,
        ResourcesInverseIndicesLists,
        QueryIndicesLists,
        ResourceViewsIndicesLists,
        DisjointIndicesLists,
        EntryIndicesLists,
        EntryViewsFilterIndicesLists,
    >,
{
    type QueryIndicesLists = QueryIndicesLists;
    type ResourceViewsIndicesLists = ResourceViewsIndicesLists;
    type DisjointIndicesLists = DisjointIndicesLists;
    type EntryIndicesLists = EntryIndicesLists;
    type EntryViewsFilterIndicesLists = EntryViewsFilterIndicesLists;
    type Stages = <T as Scheduler<
        'a,
        R,
        Resources,
        I,
        P,
        RI,
        MergeParametersList,
        ResourcesIndicesLists,
        ResourcesContainmentsLists,
        ResourcesInverseIndicesLists,
        QueryIndicesLists,
        ResourceViewsIndicesLists,
        DisjointIndicesLists,
        EntryIndicesLists,
        EntryViewsFilterIndicesLists,
    >>::Stages;

    #[inline]
    fn as_stages(&'a mut self) -> Self::Stages {
        self.as_stages()
    }
}
