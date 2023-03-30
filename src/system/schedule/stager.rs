use crate::{
    hlist::define_null,
    query::view,
    registry::{
        contains::views::{
            ContainsViewsOuter,
            Sealed as ContainsViewsSealed,
        },
        ContainsQuery,
        Registry,
    },
    system::{
        schedule::{
            claim::{
                decision,
                Claims,
                Merger,
            },
            stage,
            task,
            Stage,
            Task,
        },
        ParSystem,
        System,
    },
};

define_null!();

pub trait Stager<
    'a,
    R,
    Resources,
    C,
    I,
    P,
    RI,
    MergeParameters,
    ResourcesClaims,
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
> where
    R: Registry,
{
    type Stage: Stage<
        'a,
        R,
        Resources,
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
    >;
    type Remainder;

    fn extract_stage(&'a mut self) -> (Self::Stage, &'a mut Self::Remainder);
}

impl<'a, R, Resources, C, ResourcesClaims>
    Stager<
        'a,
        R,
        Resources,
        C,
        Null,
        Null,
        Null,
        Null,
        ResourcesClaims,
        Null,
        Null,
        Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
    > for task::Null
where
    R: Registry,
{
    type Stage = stage::Null;
    type Remainder = task::Null;

    #[inline]
    fn extract_stage(&'a mut self) -> (Self::Stage, &'a mut Self::Remainder) {
        (stage::Null, self)
    }
}

impl<
        'a,
        R,
        Resources,
        T,
        U,
        C,
        I,
        IS,
        P,
        PS,
        RI,
        RIS,
        P0, P1, I0, I1, Q0, Q1, MergeContainments, MergeParameters,
        ResourcesClaims,
        ResourcesIndices,
        ResourcesIndicesList,
        ResourcesContainments,
        ResourcesContainmentsList,
        ResourcesInverseIndices,
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
    >
    Stager<
        'a,
        R,
        Resources,
        C,
        (I, IS),
        (P, PS),
        (RI, RIS),
        ((P0, P1, I0, I1, Q0, Q1, MergeContainments), MergeParameters),
        ResourcesClaims,
        (ResourcesIndices, ResourcesIndicesList),
        (ResourcesContainments, ResourcesContainmentsList),
        (ResourcesInverseIndices, ResourcesInverseIndicesList),
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
    > for (task::System<T>, U)
where
    T::EntryViews<'a>: view::Views<'a>,
    R: ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0> + ContainsViewsSealed<'a, T::EntryViews<'a>, P1, I1, Q1>,
    <R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable: view::Merge<<<R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable as ContainsViewsOuter<'a, T::Views<'a>, P0, I0, Q0>>::Canonical, <<R as ContainsViewsSealed<'a, T::EntryViews<'a>, P1, I1, Q1>>::Viewable as ContainsViewsOuter<'a, T::EntryViews<'a>, P1, I1, Q1>>::Canonical, MergeContainments>,
    <R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable: ContainsViewsOuter<'a, T::Views<'a>, P0, I0, Q0>,
    <R as ContainsViewsSealed<'a, T::EntryViews<'a>, P1, I1, Q1>>::Viewable: ContainsViewsOuter<'a, T::EntryViews<'a>, P1, I1, Q1>,
    T: System + Send,
    C: Claims<'a, <<R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable as view::Merge<<<R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable as ContainsViewsOuter<'a, T::Views<'a>, P0, I0, Q0>>::Canonical, <<R as ContainsViewsSealed<'a, T::EntryViews<'a>, P1, I1, Q1>>::Viewable as ContainsViewsOuter<'a, T::EntryViews<'a>, P1, I1, Q1>>::Canonical, MergeContainments>>::Merged, I, P, R, RI>,
    ResourcesClaims: Claims<
        'a,
        T::ResourceViews<'a>,
        ResourcesIndices,
        ResourcesContainments,
        Resources,
        ResourcesInverseIndices,
    >,
    (
        <C as Claims<'a, <<R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable as view::Merge<<<R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable as ContainsViewsOuter<'a, T::Views<'a>, P0, I0, Q0>>::Canonical, <<R as ContainsViewsSealed<'a, T::EntryViews<'a>, P1, I1, Q1>>::Viewable as ContainsViewsOuter<'a, T::EntryViews<'a>, P1, I1, Q1>>::Canonical, MergeContainments>>::Merged, I, P, R, RI>>::Decision,
        <ResourcesClaims as Claims<
            'a,
            T::ResourceViews<'a>,
            ResourcesIndices,
            ResourcesContainments,
            Resources,
            ResourcesInverseIndices,
        >>::Decision,
    ): Merger,
    (task::System<T>, U): Cutoff<
        'a,
        R,
        Resources,
        <(
            <C as Claims<'a, <<R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable as view::Merge<<<R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable as ContainsViewsOuter<'a, T::Views<'a>, P0, I0, Q0>>::Canonical, <<R as ContainsViewsSealed<'a, T::EntryViews<'a>, P1, I1, Q1>>::Viewable as ContainsViewsOuter<'a, T::EntryViews<'a>, P1, I1, Q1>>::Canonical, MergeContainments>>::Merged, I, P, R, RI>>::Decision,
            <ResourcesClaims as Claims<
                'a,
                T::ResourceViews<'a>,
                ResourcesIndices,
                ResourcesContainments,
                Resources,
                ResourcesInverseIndices,
            >>::Decision,
        ) as Merger>::Decision,
        (<<R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable as view::Merge<<<R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable as ContainsViewsOuter<'a, T::Views<'a>, P0, I0, Q0>>::Canonical, <<R as ContainsViewsSealed<'a, T::EntryViews<'a>, P1, I1, Q1>>::Viewable as ContainsViewsOuter<'a, T::EntryViews<'a>, P1, I1, Q1>>::Canonical, MergeContainments>>::Merged, C),
        IS,
        PS,
        RIS,
        MergeParameters,
        (T::ResourceViews<'a>, ResourcesClaims),
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
{
    type Stage = <(task::System<T>, U) as Cutoff<
        'a,
        R,
        Resources,
        <(
            <C as Claims<'a, <<R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable as view::Merge<<<R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable as ContainsViewsOuter<'a, T::Views<'a>, P0, I0, Q0>>::Canonical, <<R as ContainsViewsSealed<'a, T::EntryViews<'a>, P1, I1, Q1>>::Viewable as ContainsViewsOuter<'a, T::EntryViews<'a>, P1, I1, Q1>>::Canonical, MergeContainments>>::Merged, I, P, R, RI>>::Decision,
            <ResourcesClaims as Claims<
                'a,
                T::ResourceViews<'a>,
                ResourcesIndices,
                ResourcesContainments,
                Resources,
                ResourcesInverseIndices,
            >>::Decision,
        ) as Merger>::Decision,
        (<<R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable as view::Merge<<<R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable as ContainsViewsOuter<'a, T::Views<'a>, P0, I0, Q0>>::Canonical, <<R as ContainsViewsSealed<'a, T::EntryViews<'a>, P1, I1, Q1>>::Viewable as ContainsViewsOuter<'a, T::EntryViews<'a>, P1, I1, Q1>>::Canonical, MergeContainments>>::Merged, C),
        IS,
        PS,
        RIS,
        MergeParameters,
        (T::ResourceViews<'a>, ResourcesClaims),
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
    >>::Stage;
    type Remainder = <(task::System<T>, U) as Cutoff<
        'a,
        R,
        Resources,
        <(
            <C as Claims<'a, <<R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable as view::Merge<<<R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable as ContainsViewsOuter<'a, T::Views<'a>, P0, I0, Q0>>::Canonical, <<R as ContainsViewsSealed<'a, T::EntryViews<'a>, P1, I1, Q1>>::Viewable as ContainsViewsOuter<'a, T::EntryViews<'a>, P1, I1, Q1>>::Canonical, MergeContainments>>::Merged, I, P, R, RI>>::Decision,
            <ResourcesClaims as Claims<
                'a,
                T::ResourceViews<'a>,
                ResourcesIndices,
                ResourcesContainments,
                Resources,
                ResourcesInverseIndices,
            >>::Decision,
        ) as Merger>::Decision,
        (<<R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable as view::Merge<<<R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable as ContainsViewsOuter<'a, T::Views<'a>, P0, I0, Q0>>::Canonical, <<R as ContainsViewsSealed<'a, T::EntryViews<'a>, P1, I1, Q1>>::Viewable as ContainsViewsOuter<'a, T::EntryViews<'a>, P1, I1, Q1>>::Canonical, MergeContainments>>::Merged, C),
        IS,
        PS,
        RIS,
        MergeParameters,
        (T::ResourceViews<'a>, ResourcesClaims),
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
    >>::Remainder;

    #[inline]
    fn extract_stage(&'a mut self) -> (Self::Stage, &'a mut Self::Remainder) {
        self.cutoff_stage()
    }
}

impl<
        'a,
        R,
        Resources,
        T,
        U,
        C,
        I,
        IS,
        P,
        PS,
        RI,
        RIS,
        P0, P1, I0, I1, Q0, Q1, MergeContainments, MergeParameters,
        ResourcesClaims,
        ResourcesIndices,
        ResourcesIndicesList,
        ResourcesContainments,
        ResourcesContainmentsList,
        ResourcesInverseIndices,
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
    >
    Stager<
        'a,
        R,
        Resources,
        C,
        (I, IS),
        (P, PS),
        (RI, RIS),
        ((P0, P1, I0, I1, Q0, Q1, MergeContainments), MergeParameters),
        ResourcesClaims,
        (ResourcesIndices, ResourcesIndicesList),
        (ResourcesContainments, ResourcesContainmentsList),
        (ResourcesInverseIndices, ResourcesInverseIndicesList),
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
    > for (task::ParSystem<T>, U)
where
    T::EntryViews<'a>: view::Views<'a>,
    R: ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0> + ContainsViewsSealed<'a, T::EntryViews<'a>, P1, I1, Q1>,
    <R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable: view::Merge<<<R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable as ContainsViewsOuter<'a, T::Views<'a>, P0, I0, Q0>>::Canonical, <<R as ContainsViewsSealed<'a, T::EntryViews<'a>, P1, I1, Q1>>::Viewable as ContainsViewsOuter<'a, T::EntryViews<'a>, P1, I1, Q1>>::Canonical, MergeContainments>,
    <R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable: ContainsViewsOuter<'a, T::Views<'a>, P0, I0, Q0>,
    <R as ContainsViewsSealed<'a, T::EntryViews<'a>, P1, I1, Q1>>::Viewable: ContainsViewsOuter<'a, T::EntryViews<'a>, P1, I1, Q1>,
    T: ParSystem + Send,
    C: Claims<'a, <<R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable as view::Merge<<<R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable as ContainsViewsOuter<'a, T::Views<'a>, P0, I0, Q0>>::Canonical, <<R as ContainsViewsSealed<'a, T::EntryViews<'a>, P1, I1, Q1>>::Viewable as ContainsViewsOuter<'a, T::EntryViews<'a>, P1, I1, Q1>>::Canonical, MergeContainments>>::Merged, I, P, R, RI>,
    ResourcesClaims: Claims<
        'a,
        T::ResourceViews<'a>,
        ResourcesIndices,
        ResourcesContainments,
        Resources,
        ResourcesInverseIndices,
    >,
    (
        <C as Claims<'a, <<R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable as view::Merge<<<R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable as ContainsViewsOuter<'a, T::Views<'a>, P0, I0, Q0>>::Canonical, <<R as ContainsViewsSealed<'a, T::EntryViews<'a>, P1, I1, Q1>>::Viewable as ContainsViewsOuter<'a, T::EntryViews<'a>, P1, I1, Q1>>::Canonical, MergeContainments>>::Merged, I, P, R, RI>>::Decision,
        <ResourcesClaims as Claims<
            'a,
            T::ResourceViews<'a>,
            ResourcesIndices,
            ResourcesContainments,
            Resources,
            ResourcesInverseIndices,
        >>::Decision,
    ): Merger,
    (task::ParSystem<T>, U): Cutoff<
        'a,
        R,
        Resources,
        <(
            <C as Claims<'a, <<R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable as view::Merge<<<R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable as ContainsViewsOuter<'a, T::Views<'a>, P0, I0, Q0>>::Canonical, <<R as ContainsViewsSealed<'a, T::EntryViews<'a>, P1, I1, Q1>>::Viewable as ContainsViewsOuter<'a, T::EntryViews<'a>, P1, I1, Q1>>::Canonical, MergeContainments>>::Merged, I, P, R, RI>>::Decision,
            <ResourcesClaims as Claims<
                'a,
                T::ResourceViews<'a>,
                ResourcesIndices,
                ResourcesContainments,
                Resources,
                ResourcesInverseIndices,
            >>::Decision,
        ) as Merger>::Decision,
        (<<R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable as view::Merge<<<R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable as ContainsViewsOuter<'a, T::Views<'a>, P0, I0, Q0>>::Canonical, <<R as ContainsViewsSealed<'a, T::EntryViews<'a>, P1, I1, Q1>>::Viewable as ContainsViewsOuter<'a, T::EntryViews<'a>, P1, I1, Q1>>::Canonical, MergeContainments>>::Merged, C),
        IS,
        PS,
        RIS,
        MergeParameters,
        (T::ResourceViews<'a>, ResourcesClaims),
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
{
    type Stage = <(task::ParSystem<T>, U) as Cutoff<
        'a,
        R,
        Resources,
        <(
            <C as Claims<'a, <<R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable as view::Merge<<<R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable as ContainsViewsOuter<'a, T::Views<'a>, P0, I0, Q0>>::Canonical, <<R as ContainsViewsSealed<'a, T::EntryViews<'a>, P1, I1, Q1>>::Viewable as ContainsViewsOuter<'a, T::EntryViews<'a>, P1, I1, Q1>>::Canonical, MergeContainments>>::Merged, I, P, R, RI>>::Decision,
            <ResourcesClaims as Claims<
                'a,
                T::ResourceViews<'a>,
                ResourcesIndices,
                ResourcesContainments,
                Resources,
                ResourcesInverseIndices,
            >>::Decision,
        ) as Merger>::Decision,
        (<<R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable as view::Merge<<<R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable as ContainsViewsOuter<'a, T::Views<'a>, P0, I0, Q0>>::Canonical, <<R as ContainsViewsSealed<'a, T::EntryViews<'a>, P1, I1, Q1>>::Viewable as ContainsViewsOuter<'a, T::EntryViews<'a>, P1, I1, Q1>>::Canonical, MergeContainments>>::Merged, C),
        IS,
        PS,
        RIS,
        MergeParameters,
        (T::ResourceViews<'a>, ResourcesClaims),
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
    >>::Stage;
    type Remainder = <(task::ParSystem<T>, U) as Cutoff<
        'a,
        R,
        Resources,
        <(
            <C as Claims<'a, <<R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable as view::Merge<<<R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable as ContainsViewsOuter<'a, T::Views<'a>, P0, I0, Q0>>::Canonical, <<R as ContainsViewsSealed<'a, T::EntryViews<'a>, P1, I1, Q1>>::Viewable as ContainsViewsOuter<'a, T::EntryViews<'a>, P1, I1, Q1>>::Canonical, MergeContainments>>::Merged, I, P, R, RI>>::Decision,
            <ResourcesClaims as Claims<
                'a,
                T::ResourceViews<'a>,
                ResourcesIndices,
                ResourcesContainments,
                Resources,
                ResourcesInverseIndices,
            >>::Decision,
        ) as Merger>::Decision,
        (<<R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable as view::Merge<<<R as ContainsViewsSealed<'a, T::Views<'a>, P0, I0, Q0>>::Viewable as ContainsViewsOuter<'a, T::Views<'a>, P0, I0, Q0>>::Canonical, <<R as ContainsViewsSealed<'a, T::EntryViews<'a>, P1, I1, Q1>>::Viewable as ContainsViewsOuter<'a, T::EntryViews<'a>, P1, I1, Q1>>::Canonical, MergeContainments>>::Merged, C),
        IS,
        PS,
        RIS,
        MergeParameters,
        (T::ResourceViews<'a>, ResourcesClaims),
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
    >>::Remainder;

    #[inline]
    fn extract_stage(&'a mut self) -> (Self::Stage, &'a mut Self::Remainder) {
        self.cutoff_stage()
    }
}

pub trait Cutoff<
    'a,
    R,
    Resources,
    D,
    C,
    I,
    P,
    RI,
    MergeParameters,
    ResourcesClaims,
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
> where
    R: Registry,
{
    type Stage: Stage<
        'a,
        R,
        Resources,
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
    >;
    type Remainder;

    fn cutoff_stage(&'a mut self) -> (Self::Stage, &'a mut Self::Remainder);
}

impl<'a, R, Resources, T, C, ResourcesClaims>
    Cutoff<
        'a,
        R,
        Resources,
        decision::Cut,
        C,
        Null,
        Null,
        Null,
        Null,
        ResourcesClaims,
        Null,
        Null,
        Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
        stage::Null,
    > for T
where
    R: Registry,
    T: 'a,
{
    type Stage = stage::Null;
    type Remainder = T;

    #[inline]
    fn cutoff_stage(&'a mut self) -> (Self::Stage, &'a mut Self::Remainder) {
        (stage::Null, self)
    }
}

impl<
        'a,
        R,
        Resources,
        T,
        U,
        C,
        I,
        P,
        RI,
        MergeParameters,
        ResourcesClaims,
        ResourcesIndicesList,
        ResourcesContainmentsList,
        ResourcesInverseIndicesList,
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
        ResourceViewsContainments,
        ResourceViewsContainmentsList,
        ResourceViewsIndices,
        ResourceViewsIndicesList,
        ResourceViewsCanonicalContainments,
        ResourceViewsCanonicalContainmentsList,
        ResourceViewsReshapeIndices,
        ResourceViewsReshapeIndicesList,
        EntryViewsContainments,
        EntryViewsContainmentsList,
        EntryViewsIndices,
        EntryViewsIndicesList,
        EntryViewsReshapeIndices,
        EntryViewsReshapeIndicesList,
        EntryViewsInverseIndices,
        EntryViewsInverseIndicesList,
        EntryViewsOppositeContainments,
        EntryViewsOppositeContainmentsList,
        EntryViewsOppositeIndices,
        EntryViewsOppositeIndicesList,
        EntryViewsOppositeReshapeIndices,
        EntryViewsOppositeReshapeIndicesList,
        EntryViewsOppositeInverseIndices,
        EntryViewsOppositeInverseIndicesList,
        EntryContainments,
        EntryContainmentsList,
        EntryIndices,
        EntryIndicesList,
        EntryReshapeIndices,
        EntryReshapeIndicesList,
    >
    Cutoff<
        'a,
        R,
        Resources,
        decision::Append,
        C,
        I,
        P,
        RI,
        MergeParameters,
        ResourcesClaims,
        ResourcesIndicesList,
        ResourcesContainmentsList,
        ResourcesInverseIndicesList,
        (SFI, SFIS),
        (SVI, SVIS),
        (SP, SPS),
        (SI, SIS),
        (SQ, SQS),
        (ResourceViewsContainments, ResourceViewsContainmentsList),
        (ResourceViewsIndices, ResourceViewsIndicesList),
        (
            ResourceViewsCanonicalContainments,
            ResourceViewsCanonicalContainmentsList,
        ),
        (ResourceViewsReshapeIndices, ResourceViewsReshapeIndicesList),
        (EntryViewsContainments, EntryViewsContainmentsList),
        (EntryViewsIndices, EntryViewsIndicesList),
        (EntryViewsReshapeIndices, EntryViewsReshapeIndicesList),
        (EntryViewsInverseIndices, EntryViewsInverseIndicesList),
        (
            EntryViewsOppositeContainments,
            EntryViewsOppositeContainmentsList,
        ),
        (EntryViewsOppositeIndices, EntryViewsOppositeIndicesList),
        (
            EntryViewsOppositeReshapeIndices,
            EntryViewsOppositeReshapeIndicesList,
        ),
        (
            EntryViewsOppositeInverseIndices,
            EntryViewsOppositeInverseIndicesList,
        ),
        (EntryContainments, EntryContainmentsList),
        (EntryIndices, EntryIndicesList),
        (EntryReshapeIndices, EntryReshapeIndicesList),
    > for (T, U)
where
    R: ContainsQuery<'a, T::Filter, SFI, T::Views, SVI, SP, SI, SQ>,
    Resources: 'a,
    T: Task<
            'a,
            R,
            Resources,
            SFI,
            SVI,
            SP,
            SI,
            SQ,
            ResourceViewsContainments,
            ResourceViewsIndices,
            ResourceViewsCanonicalContainments,
            ResourceViewsReshapeIndices,
            EntryViewsContainments,
            EntryViewsIndices,
            EntryViewsReshapeIndices,
            EntryViewsInverseIndices,
            EntryViewsOppositeContainments,
            EntryViewsOppositeIndices,
            EntryViewsOppositeReshapeIndices,
            EntryViewsOppositeInverseIndices,
            EntryContainments,
            EntryIndices,
            EntryReshapeIndices,
        > + Send
        + 'a,
    U: Stager<
        'a,
        R,
        Resources,
        C,
        I,
        P,
        RI,
        MergeParameters,
        ResourcesClaims,
        ResourcesIndicesList,
        ResourcesContainmentsList,
        ResourcesInverseIndicesList,
        SFIS,
        SVIS,
        SPS,
        SIS,
        SQS,
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
{
    type Stage = (
        &'a mut T,
        <U as Stager<
            'a,
            R,
            Resources,
            C,
            I,
            P,
            RI,
            MergeParameters,
            ResourcesClaims,
            ResourcesIndicesList,
            ResourcesContainmentsList,
            ResourcesInverseIndicesList,
            SFIS,
            SVIS,
            SPS,
            SIS,
            SQS,
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
    );
    type Remainder = <U as Stager<
        'a,
        R,
        Resources,
        C,
        I,
        P,
        RI,
        MergeParameters,
        ResourcesClaims,
        ResourcesIndicesList,
        ResourcesContainmentsList,
        ResourcesInverseIndicesList,
        SFIS,
        SVIS,
        SPS,
        SIS,
        SQS,
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
    >>::Remainder;

    #[inline]
    fn cutoff_stage(&'a mut self) -> (Self::Stage, &'a mut Self::Remainder) {
        let (stage, remainder) = self.1.extract_stage();
        ((&mut self.0, stage), remainder)
    }
}
