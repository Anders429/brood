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
    >
    Sealed<
        'a,
        R,
        Resources,
        I,
        P,
        RI,
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
    >,
{
    type Stages = <T as Scheduler<
        'a,
        R,
        Resources,
        I,
        P,
        RI,
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
    >>::Stages;

    #[inline]
    fn as_stages(&'a mut self) -> Self::Stages {
        self.as_stages()
    }
}
