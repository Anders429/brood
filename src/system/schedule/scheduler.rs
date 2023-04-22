use crate::{
    hlist::define_null,
    registry::Registry,
    system::schedule::{
        claim,
        stages,
        task,
        Stager,
        Stages,
    },
};

define_null!();

pub trait Scheduler<
    'a,
    R,
    Resources,
    I,
    P,
    RI,
    MergeParamtersList,
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

impl<'a, R, Resources>
    Scheduler<
        'a,
        R,
        Resources,
        Null,
        Null,
        Null,
        stages::Null,
        stages::Null,
        stages::Null,
        stages::Null,
        stages::Null,
        stages::Null,
        stages::Null,
        stages::Null,
    > for task::Null
where
    R: Registry,
{
    type Stages = stages::Null;

    #[inline]
    fn as_stages(&'a mut self) -> Self::Stages {
        stages::Null
    }
}

impl<
        'a,
        R,
        Resources,
        T,
        U,
        I,
        IS,
        P,
        PS,
        RI,
        RIS,
        MergeParameters,
        MergeParametersList,
        ResourcesIndicesList,
        ResourcesIndicesLists,
        ResourcesContainmentsList,
        ResourcesContainmentsLists,
        ResourcesInverseIndicesList,
        ResourcesInverseIndicesLists,
        QueryIndicesList,
        QueryIndicesLists,
        ResourceViewsIndicesList,
        ResourceViewsIndicesLists,
        DisjointIndicesList,
        DisjointIndicesLists,
        EntryIndicesList,
        EntryIndicesLists,
    >
    Scheduler<
        'a,
        R,
        Resources,
        (I, IS),
        (P, PS),
        (RI, RIS),
        (MergeParameters, MergeParametersList),
        (ResourcesIndicesList, ResourcesIndicesLists),
        (ResourcesContainmentsList, ResourcesContainmentsLists),
        (ResourcesInverseIndicesList, ResourcesInverseIndicesLists),
        (QueryIndicesList, QueryIndicesLists),
        (ResourceViewsIndicesList, ResourceViewsIndicesLists),
        (DisjointIndicesList, DisjointIndicesLists),
        (EntryIndicesList, EntryIndicesLists),
    > for (T, U)
where
    (T, U): Stager<
        'a,
        R,
        Resources,
        claim::Null,
        I,
        P,
        RI,
        MergeParameters,
        claim::Null,
        ResourcesIndicesList,
        ResourcesContainmentsList,
        ResourcesInverseIndicesList,
        QueryIndicesList,
        ResourceViewsIndicesList,
        DisjointIndicesList,
        EntryIndicesList,
    >,
    <(T, U) as Stager<
        'a,
        R,
        Resources,
        claim::Null,
        I,
        P,
        RI,
        MergeParameters,
        claim::Null,
        ResourcesIndicesList,
        ResourcesContainmentsList,
        ResourcesInverseIndicesList,
        QueryIndicesList,
        ResourceViewsIndicesList,
        DisjointIndicesList,
        EntryIndicesList,
    >>::Remainder: Scheduler<
        'a,
        R,
        Resources,
        IS,
        PS,
        RIS,
        MergeParametersList,
        ResourcesIndicesLists,
        ResourcesContainmentsLists,
        ResourcesInverseIndicesLists,
        QueryIndicesLists,
        ResourceViewsIndicesLists,
        DisjointIndicesLists,
        EntryIndicesLists,
    >,
    R: Registry + 'a,
    Resources: 'a,
    I: 'a,
    P: 'a,
    RI: 'a,
    MergeParameters: 'a,
    ResourcesIndicesList: 'a,
    ResourcesContainmentsList: 'a,
    ResourcesInverseIndicesList: 'a,
    QueryIndicesList: 'a,
    ResourceViewsIndicesList: 'a,
    DisjointIndicesList: 'a,
    EntryIndicesList: 'a,
{
    type Stages = (
        <(T, U) as Stager<
            'a,
            R,
            Resources,
            claim::Null,
            I,
            P,
            RI,
            MergeParameters,
            claim::Null,
            ResourcesIndicesList,
            ResourcesContainmentsList,
            ResourcesInverseIndicesList,
            QueryIndicesList,
            ResourceViewsIndicesList,
            DisjointIndicesList,
            EntryIndicesList,
        >>::Stage,
        <<(T, U) as Stager<
            'a,
            R,
            Resources,
            claim::Null,
            I,
            P,
            RI,
            MergeParameters,
            claim::Null,
            ResourcesIndicesList,
            ResourcesContainmentsList,
            ResourcesInverseIndicesList,
            QueryIndicesList,
            ResourceViewsIndicesList,
            DisjointIndicesList,
            EntryIndicesList,
        >>::Remainder as Scheduler<
            'a,
            R,
            Resources,
            IS,
            PS,
            RIS,
            MergeParametersList,
            ResourcesIndicesLists,
            ResourcesContainmentsLists,
            ResourcesInverseIndicesLists,
            QueryIndicesLists,
            ResourceViewsIndicesLists,
            DisjointIndicesLists,
            EntryIndicesLists,
        >>::Stages,
    );

    #[inline]
    fn as_stages(&'a mut self) -> Self::Stages {
        let (stage, remainder) = self.extract_stage();
        (stage, remainder.as_stages())
    }
}
