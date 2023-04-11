use crate::{
    registry::Registry,
    system::schedule::{
        Scheduler,
        Stages,
    },
};

pub trait Sealed<
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
> where
    R: Registry,
{
    type Stages: Stages<
        'a,
        R,
        Resources,
        QueryIndicesLists,
        ResourceViewsIndicesLists,
        DisjointIndicesLists,
        EntryIndicesLists,
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
    >
    Sealed<
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
    >,
{
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
    >>::Stages;

    #[inline]
    fn as_stages(&'a mut self) -> Self::Stages {
        self.as_stages()
    }
}
