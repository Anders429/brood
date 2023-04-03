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
    SFI,
    SVI,
    SP,
    SI,
    SQ,
    ResourceViewsContainmentsLists,
    ResourceViewsIndicesLists,
    ResourceViewsCanonicalContainmentsLists,
    ResourceViewsReshapeIndicesLists,
    EntryViewsContainmentsLists,
    EntryViewsIndicesLists,
    EntryViewsReshapeIndicesLists,
    EntryViewsInverseIndicesLists,
    EntryViewsOppositeContainmentsLists,
    EntryViewsOppositeIndicesLists,
    EntryViewsOppositeReshapeIndicesLists,
    EntryViewsOppositeInverseIndicesLists,
    EntryContainmentsLists,
    EntryIndicesLists,
    EntryReshapeIndicesLists,
> where
    R: Registry,
{
    type Stages: Stages<
        'a,
        R,
        Resources,
        SFI,
        SVI,
        SP,
        SI,
        SQ,
        ResourceViewsContainmentsLists,
        ResourceViewsIndicesLists,
        ResourceViewsCanonicalContainmentsLists,
        ResourceViewsReshapeIndicesLists,
        EntryViewsContainmentsLists,
        EntryViewsIndicesLists,
        EntryViewsReshapeIndicesLists,
        EntryViewsInverseIndicesLists,
        EntryViewsOppositeContainmentsLists,
        EntryViewsOppositeIndicesLists,
        EntryViewsOppositeReshapeIndicesLists,
        EntryViewsOppositeInverseIndicesLists,
        EntryContainmentsLists,
        EntryIndicesLists,
        EntryReshapeIndicesLists,
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
        Null,
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
        stages::Null,
        stages::Null,
        stages::Null,
        stages::Null,
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
        SFI,
        SFIS,
        SVI,
        SVIS,
        SP,
        SPS,
        SI,
        SIS,
        SQ,
        SQS,
        ResourceViewsContainmentsList,
        ResourceViewsContainmentsLists,
        ResourceViewsIndicesList,
        ResourceViewsIndicesLists,
        ResourceViewsCanonicalContainmentsList,
        ResourceViewsCanonicalContainmentsLists,
        ResourceViewsReshapeIndicesList,
        ResourceViewsReshapeIndicesLists,
        EntryViewsContainmentsList,
        EntryViewsContainmentsLists,
        EntryViewsIndicesList,
        EntryViewsIndicesLists,
        EntryViewsReshapeIndicesList,
        EntryViewsReshapeIndicesLists,
        EntryViewsInverseIndicesList,
        EntryViewsInverseIndicesLists,
        EntryViewsOppositeContainmentsList,
        EntryViewsOppositeContainmentsLists,
        EntryViewsOppositeIndicesList,
        EntryViewsOppositeIndicesLists,
        EntryViewsOppositeReshapeIndicesList,
        EntryViewsOppositeReshapeIndicesLists,
        EntryViewsOppositeInverseIndicesList,
        EntryViewsOppositeInverseIndicesLists,
        EntryContainmentsList,
        EntryContainmentsLists,
        EntryIndicesList,
        EntryIndicesLists,
        EntryReshapeIndicesList,
        EntryReshapeIndicesLists,
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
        (SFI, SFIS),
        (SVI, SVIS),
        (SP, SPS),
        (SI, SIS),
        (SQ, SQS),
        (
            ResourceViewsContainmentsList,
            ResourceViewsContainmentsLists,
        ),
        (ResourceViewsIndicesList, ResourceViewsIndicesLists),
        (
            ResourceViewsCanonicalContainmentsList,
            ResourceViewsCanonicalContainmentsLists,
        ),
        (
            ResourceViewsReshapeIndicesList,
            ResourceViewsReshapeIndicesLists,
        ),
        (EntryViewsContainmentsList, EntryViewsContainmentsLists),
        (EntryViewsIndicesList, EntryViewsIndicesLists),
        (EntryViewsReshapeIndicesList, EntryViewsReshapeIndicesLists),
        (EntryViewsInverseIndicesList, EntryViewsInverseIndicesLists),
        (
            EntryViewsOppositeContainmentsList,
            EntryViewsOppositeContainmentsLists,
        ),
        (
            EntryViewsOppositeIndicesList,
            EntryViewsOppositeIndicesLists,
        ),
        (
            EntryViewsOppositeReshapeIndicesList,
            EntryViewsOppositeReshapeIndicesLists,
        ),
        (
            EntryViewsOppositeInverseIndicesList,
            EntryViewsOppositeInverseIndicesLists,
        ),
        (EntryContainmentsList, EntryContainmentsLists),
        (EntryIndicesList, EntryIndicesLists),
        (EntryReshapeIndicesList, EntryReshapeIndicesLists),
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
        SFI,
        SVI,
        SP,
        SI,
        SQ,
        ResourceViewsContainmentsList,
        ResourceViewsIndicesList,
        ResourceViewsCanonicalContainmentsList,
        ResourceViewsReshapeIndicesList,
        EntryViewsContainmentsList,
        EntryViewsIndicesList,
        EntryViewsReshapeIndicesList,
        EntryViewsInverseIndicesList,
        EntryViewsOppositeContainmentsList,
        EntryViewsOppositeIndicesList,
        EntryViewsOppositeReshapeIndicesList,
        EntryViewsOppositeInverseIndicesList,
        EntryContainmentsList,
        EntryIndicesList,
        EntryReshapeIndicesList,
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
        SFI,
        SVI,
        SP,
        SI,
        SQ,
        ResourceViewsContainmentsList,
        ResourceViewsIndicesList,
        ResourceViewsCanonicalContainmentsList,
        ResourceViewsReshapeIndicesList,
        EntryViewsContainmentsList,
        EntryViewsIndicesList,
        EntryViewsReshapeIndicesList,
        EntryViewsInverseIndicesList,
        EntryViewsOppositeContainmentsList,
        EntryViewsOppositeIndicesList,
        EntryViewsOppositeReshapeIndicesList,
        EntryViewsOppositeInverseIndicesList,
        EntryContainmentsList,
        EntryIndicesList,
        EntryReshapeIndicesList,
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
        SFIS,
        SVIS,
        SPS,
        SIS,
        SQS,
        ResourceViewsContainmentsLists,
        ResourceViewsIndicesLists,
        ResourceViewsCanonicalContainmentsLists,
        ResourceViewsReshapeIndicesLists,
        EntryViewsContainmentsLists,
        EntryViewsIndicesLists,
        EntryViewsReshapeIndicesLists,
        EntryViewsInverseIndicesLists,
        EntryViewsOppositeContainmentsLists,
        EntryViewsOppositeIndicesLists,
        EntryViewsOppositeReshapeIndicesLists,
        EntryViewsOppositeInverseIndicesLists,
        EntryContainmentsLists,
        EntryIndicesLists,
        EntryReshapeIndicesLists,
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
    SFI: 'a,
    SVI: 'a,
    SP: 'a,
    SI: 'a,
    SQ: 'a,
    ResourceViewsContainmentsList: 'a,
    ResourceViewsIndicesList: 'a,
    ResourceViewsCanonicalContainmentsList: 'a,
    ResourceViewsReshapeIndicesList: 'a,
    EntryViewsContainmentsList: 'a,
    EntryViewsIndicesList: 'a,
    EntryViewsReshapeIndicesList: 'a,
    EntryViewsInverseIndicesList: 'a,
    EntryViewsOppositeContainmentsList: 'a,
    EntryViewsOppositeIndicesList: 'a,
    EntryViewsOppositeReshapeIndicesList: 'a,
    EntryViewsOppositeInverseIndicesList: 'a,
    EntryContainmentsList: 'a,
    EntryIndicesList: 'a,
    EntryReshapeIndicesList: 'a,
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
            SFI,
            SVI,
            SP,
            SI,
            SQ,
            ResourceViewsContainmentsList,
            ResourceViewsIndicesList,
            ResourceViewsCanonicalContainmentsList,
            ResourceViewsReshapeIndicesList,
            EntryViewsContainmentsList,
            EntryViewsIndicesList,
            EntryViewsReshapeIndicesList,
            EntryViewsInverseIndicesList,
            EntryViewsOppositeContainmentsList,
            EntryViewsOppositeIndicesList,
            EntryViewsOppositeReshapeIndicesList,
            EntryViewsOppositeInverseIndicesList,
            EntryContainmentsList,
            EntryIndicesList,
            EntryReshapeIndicesList,
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
            SFI,
            SVI,
            SP,
            SI,
            SQ,
            ResourceViewsContainmentsList,
            ResourceViewsIndicesList,
            ResourceViewsCanonicalContainmentsList,
            ResourceViewsReshapeIndicesList,
            EntryViewsContainmentsList,
            EntryViewsIndicesList,
            EntryViewsReshapeIndicesList,
            EntryViewsInverseIndicesList,
            EntryViewsOppositeContainmentsList,
            EntryViewsOppositeIndicesList,
            EntryViewsOppositeReshapeIndicesList,
            EntryViewsOppositeInverseIndicesList,
            EntryContainmentsList,
            EntryIndicesList,
            EntryReshapeIndicesList,
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
            SFIS,
            SVIS,
            SPS,
            SIS,
            SQS,
            ResourceViewsContainmentsLists,
            ResourceViewsIndicesLists,
            ResourceViewsCanonicalContainmentsLists,
            ResourceViewsReshapeIndicesLists,
            EntryViewsContainmentsLists,
            EntryViewsIndicesLists,
            EntryViewsReshapeIndicesLists,
            EntryViewsInverseIndicesLists,
            EntryViewsOppositeContainmentsLists,
            EntryViewsOppositeIndicesLists,
            EntryViewsOppositeReshapeIndicesLists,
            EntryViewsOppositeInverseIndicesLists,
            EntryContainmentsLists,
            EntryIndicesLists,
            EntryReshapeIndicesLists,
        >>::Stages,
    );

    #[inline]
    fn as_stages(&'a mut self) -> Self::Stages {
        let (stage, remainder) = self.extract_stage();
        (stage, remainder.as_stages())
    }
}
