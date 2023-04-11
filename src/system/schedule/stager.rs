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
    QueryIndicesList,
    ResourceViewsIndicesList,
    DisjointIndicesList,
    EntryIndicesList,
> where
    R: Registry,
{
    type Stage: Stage<
        'a,
        R,
        Resources,
        QueryIndicesList,
        ResourceViewsIndicesList,
        DisjointIndicesList,
        EntryIndicesList,
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
        LeftMergeIndices, RightMergeIndices, MergeContainments, MergeParameters,
        ResourcesClaims,
        ResourcesIndices,
        ResourcesIndicesList,
        ResourcesContainments,
        ResourcesContainmentsList,
        ResourcesInverseIndices,
        ResourcesInverseIndicesList,
        QueryIndicesList,
        ResourceViewsIndicesList,
        DisjointIndicesList,
        EntryIndicesList,
    >
    Stager<
        'a,
        R,
        Resources,
        C,
        (I, IS),
        (P, PS),
        (RI, RIS),
        ((LeftMergeIndices, RightMergeIndices, MergeContainments), MergeParameters),
        ResourcesClaims,
        (ResourcesIndices, ResourcesIndicesList),
        (ResourcesContainments, ResourcesContainmentsList),
        (ResourcesInverseIndices, ResourcesInverseIndicesList),
        QueryIndicesList,
        ResourceViewsIndicesList,
        DisjointIndicesList,
        EntryIndicesList,
    > for (task::System<T>, U)
where
    T::EntryViews<'a>: view::Views<'a>,
    R: ContainsViewsSealed<
        'a,
        T::Views<'a>,
        LeftMergeIndices
    > + ContainsViewsSealed<
        'a,
        T::EntryViews<'a>,
        RightMergeIndices
    >,
    <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Viewable: view::Merge<
        <<R as ContainsViewsSealed<
            'a,
            T::Views<'a>,
            LeftMergeIndices
        >>::Viewable as ContainsViewsOuter<
            'a,
            T::Views<'a>,
            <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Containments,
            <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Indices,
            <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::ReshapeIndices,
        >>::Canonical,
        <<R as ContainsViewsSealed<
            'a,
            T::EntryViews<'a>,
            RightMergeIndices
        >>::Viewable as ContainsViewsOuter<
            'a,
            T::EntryViews<'a>,
            <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Containments,
            <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Indices,
            <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::ReshapeIndices,
        >>::Canonical,
        MergeContainments
    >,
    <R as ContainsViewsSealed<
        'a,
        T::Views<'a>,
        LeftMergeIndices
    >>::Viewable: ContainsViewsOuter<
        'a,
        T::Views<'a>,
        <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Containments,
        <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Indices,
        <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::ReshapeIndices,
    >,
    <R as ContainsViewsSealed<
        'a,
        T::EntryViews<'a>,
        RightMergeIndices
    >>::Viewable: ContainsViewsOuter<
        'a,
        T::EntryViews<'a>,
        <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Containments,
        <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Indices,
        <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::ReshapeIndices,
    >,
    T: System + Send,
    C: Claims<
        'a,
        <<R as ContainsViewsSealed<
            'a,
            T::Views<'a>,
            LeftMergeIndices
        >>::Viewable as view::Merge<
            <<R as ContainsViewsSealed<
                'a,
                T::Views<'a>,
                LeftMergeIndices
            >>::Viewable as ContainsViewsOuter<
                'a,
                T::Views<'a>,
                <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Containments,
                <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Indices,
                <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::ReshapeIndices,
            >>::Canonical,
            <<R as ContainsViewsSealed<
                'a,
                T::EntryViews<'a>,
                RightMergeIndices,
            >>::Viewable as ContainsViewsOuter<
                'a,
                T::EntryViews<'a>,
                <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Containments,
                <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Indices,
                <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::ReshapeIndices,
            >>::Canonical,
            MergeContainments,
        >>::Merged,
        I,
        P,
        R,
        RI,
    >,
    ResourcesClaims: Claims<
        'a,
        T::ResourceViews<'a>,
        ResourcesIndices,
        ResourcesContainments,
        Resources,
        ResourcesInverseIndices,
    >,
    (
        <C as Claims<'a, <<R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Viewable as view::Merge<<<R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Viewable as ContainsViewsOuter<'a, T::Views<'a>, <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Containments, <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Indices, <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::ReshapeIndices>>::Canonical, <<R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Viewable as ContainsViewsOuter<'a, T::EntryViews<'a>, <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Containments, <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Indices, <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::ReshapeIndices>>::Canonical, MergeContainments>>::Merged, I, P, R, RI>>::Decision,
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
            <C as Claims<'a, <<R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Viewable as view::Merge<<<R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Viewable as ContainsViewsOuter<'a, T::Views<'a>,
            <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Containments,
            <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Indices,
            <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::ReshapeIndices,>>::Canonical, <<R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Viewable as ContainsViewsOuter<'a, T::EntryViews<'a>,
            <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Containments,
            <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Indices,
            <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::ReshapeIndices,>>::Canonical, MergeContainments>>::Merged, I, P, R, RI>>::Decision,
            <ResourcesClaims as Claims<
                'a,
                T::ResourceViews<'a>,
                ResourcesIndices,
                ResourcesContainments,
                Resources,
                ResourcesInverseIndices,
            >>::Decision,
        ) as Merger>::Decision,
        (<<R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Viewable as view::Merge<<<R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Viewable as ContainsViewsOuter<'a, T::Views<'a>,
        <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Containments,
        <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Indices,
        <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::ReshapeIndices,>>::Canonical, <<R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Viewable as ContainsViewsOuter<'a, T::EntryViews<'a>,
        <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Containments,
        <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Indices,
        <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::ReshapeIndices,>>::Canonical, MergeContainments>>::Merged, C),
        IS,
        PS,
        RIS,
        MergeParameters,
        (T::ResourceViews<'a>, ResourcesClaims),
        ResourcesIndicesList,
        ResourcesContainmentsList,
        ResourcesInverseIndicesList,
        QueryIndicesList,
        ResourceViewsIndicesList,
        DisjointIndicesList,
        EntryIndicesList,
    >,
{
    type Stage = <(task::System<T>, U) as Cutoff<
        'a,
        R,
        Resources,
        <(
            <C as Claims<'a, <<R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Viewable as view::Merge<<<R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Viewable as ContainsViewsOuter<'a, T::Views<'a>,
            <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Containments,
            <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Indices,
            <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::ReshapeIndices,>>::Canonical, <<R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Viewable as ContainsViewsOuter<'a, T::EntryViews<'a>,
            <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Containments,
            <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Indices,
            <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::ReshapeIndices,>>::Canonical, MergeContainments>>::Merged, I, P, R, RI>>::Decision,
            <ResourcesClaims as Claims<
                'a,
                T::ResourceViews<'a>,
                ResourcesIndices,
                ResourcesContainments,
                Resources,
                ResourcesInverseIndices,
            >>::Decision,
        ) as Merger>::Decision,
        (<<R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Viewable as view::Merge<<<R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Viewable as ContainsViewsOuter<'a, T::Views<'a>,
        <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Containments,
        <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Indices,
        <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::ReshapeIndices,>>::Canonical, <<R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Viewable as ContainsViewsOuter<'a, T::EntryViews<'a>,
        <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Containments,
        <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Indices,
        <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::ReshapeIndices,>>::Canonical, MergeContainments>>::Merged, C),
        IS,
        PS,
        RIS,
        MergeParameters,
        (T::ResourceViews<'a>, ResourcesClaims),
        ResourcesIndicesList,
        ResourcesContainmentsList,
        ResourcesInverseIndicesList,
        QueryIndicesList,
        ResourceViewsIndicesList,
        DisjointIndicesList,
        EntryIndicesList,
    >>::Stage;
    type Remainder = <(task::System<T>, U) as Cutoff<
        'a,
        R,
        Resources,
        <(
            <C as Claims<'a, <<R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Viewable as view::Merge<<<R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Viewable as ContainsViewsOuter<'a, T::Views<'a>,
            <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Containments,
            <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Indices,
            <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::ReshapeIndices,>>::Canonical, <<R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Viewable as ContainsViewsOuter<'a, T::EntryViews<'a>,
            <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Containments,
            <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Indices,
            <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::ReshapeIndices,>>::Canonical, MergeContainments>>::Merged, I, P, R, RI>>::Decision,
            <ResourcesClaims as Claims<
                'a,
                T::ResourceViews<'a>,
                ResourcesIndices,
                ResourcesContainments,
                Resources,
                ResourcesInverseIndices,
            >>::Decision,
        ) as Merger>::Decision,
        (<<R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Viewable as view::Merge<<<R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Viewable as ContainsViewsOuter<'a, T::Views<'a>,
        <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Containments,
        <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Indices,
        <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::ReshapeIndices,>>::Canonical, <<R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Viewable as ContainsViewsOuter<'a, T::EntryViews<'a>,
        <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Containments,
        <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Indices,
        <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::ReshapeIndices,>>::Canonical, MergeContainments>>::Merged, C),
        IS,
        PS,
        RIS,
        MergeParameters,
        (T::ResourceViews<'a>, ResourcesClaims),
        ResourcesIndicesList,
        ResourcesContainmentsList,
        ResourcesInverseIndicesList,
        QueryIndicesList,
        ResourceViewsIndicesList,
        DisjointIndicesList,
        EntryIndicesList,
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
        LeftMergeIndices, RightMergeIndices, MergeContainments, MergeParameters,
        ResourcesClaims,
        ResourcesIndices,
        ResourcesIndicesList,
        ResourcesContainments,
        ResourcesContainmentsList,
        ResourcesInverseIndices,
        ResourcesInverseIndicesList,
        QueryIndicesList,
        ResourceViewsIndicesList,
        DisjointIndicesList,
        EntryIndicesList,
    >
    Stager<
        'a,
        R,
        Resources,
        C,
        (I, IS),
        (P, PS),
        (RI, RIS),
        ((LeftMergeIndices, RightMergeIndices, MergeContainments), MergeParameters),
        ResourcesClaims,
        (ResourcesIndices, ResourcesIndicesList),
        (ResourcesContainments, ResourcesContainmentsList),
        (ResourcesInverseIndices, ResourcesInverseIndicesList),
        QueryIndicesList,
        ResourceViewsIndicesList,
        DisjointIndicesList,
        EntryIndicesList,
    > for (task::ParSystem<T>, U)
where
    T::EntryViews<'a>: view::Views<'a>,
    R: ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices> + ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>,
    <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Viewable: view::Merge<<<R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Viewable as ContainsViewsOuter<'a, T::Views<'a>,
    <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Containments,
    <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Indices,
    <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::ReshapeIndices,>>::Canonical, <<R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Viewable as ContainsViewsOuter<'a, T::EntryViews<'a>,
    <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Containments,
    <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Indices,
    <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::ReshapeIndices,>>::Canonical, MergeContainments>,
    <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Viewable: ContainsViewsOuter<'a, T::Views<'a>,
    <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Containments,
    <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Indices,
    <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::ReshapeIndices,>,
    <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Viewable: ContainsViewsOuter<'a, T::EntryViews<'a>,
    <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Containments,
    <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Indices,
    <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::ReshapeIndices,>,
    T: ParSystem + Send,
    C: Claims<'a, <<R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Viewable as view::Merge<<<R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Viewable as ContainsViewsOuter<'a, T::Views<'a>,
    <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Containments,
    <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Indices,
    <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::ReshapeIndices,>>::Canonical, <<R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Viewable as ContainsViewsOuter<'a, T::EntryViews<'a>,
    <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Containments,
    <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Indices,
    <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::ReshapeIndices,>>::Canonical, MergeContainments>>::Merged, I, P, R, RI>,
    ResourcesClaims: Claims<
        'a,
        T::ResourceViews<'a>,
        ResourcesIndices,
        ResourcesContainments,
        Resources,
        ResourcesInverseIndices,
    >,
    (
        <C as Claims<'a, <<R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Viewable as view::Merge<<<R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Viewable as ContainsViewsOuter<'a, T::Views<'a>,
        <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Containments,
        <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Indices,
        <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::ReshapeIndices,>>::Canonical, <<R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Viewable as ContainsViewsOuter<'a, T::EntryViews<'a>,
        <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Containments,
        <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Indices,
        <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::ReshapeIndices,>>::Canonical, MergeContainments>>::Merged, I, P, R, RI>>::Decision,
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
            <C as Claims<'a, <<R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Viewable as view::Merge<<<R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Viewable as ContainsViewsOuter<'a, T::Views<'a>,
            <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Containments,
            <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Indices,
            <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::ReshapeIndices,>>::Canonical, <<R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Viewable as ContainsViewsOuter<'a, T::EntryViews<'a>,
            <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Containments,
            <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Indices,
            <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::ReshapeIndices,>>::Canonical, MergeContainments>>::Merged, I, P, R, RI>>::Decision,
            <ResourcesClaims as Claims<
                'a,
                T::ResourceViews<'a>,
                ResourcesIndices,
                ResourcesContainments,
                Resources,
                ResourcesInverseIndices,
            >>::Decision,
        ) as Merger>::Decision,
        (<<R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Viewable as view::Merge<<<R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Viewable as ContainsViewsOuter<'a, T::Views<'a>,
        <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Containments,
        <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Indices,
        <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::ReshapeIndices,>>::Canonical, <<R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Viewable as ContainsViewsOuter<'a, T::EntryViews<'a>,
        <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Containments,
        <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Indices,
        <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::ReshapeIndices,>>::Canonical, MergeContainments>>::Merged, C),
        IS,
        PS,
        RIS,
        MergeParameters,
        (T::ResourceViews<'a>, ResourcesClaims),
        ResourcesIndicesList,
        ResourcesContainmentsList,
        ResourcesInverseIndicesList,
        QueryIndicesList,
        ResourceViewsIndicesList,
        DisjointIndicesList,
        EntryIndicesList,
    >,
{
    type Stage = <(task::ParSystem<T>, U) as Cutoff<
        'a,
        R,
        Resources,
        <(
            <C as Claims<'a, <<R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Viewable as view::Merge<<<R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Viewable as ContainsViewsOuter<'a, T::Views<'a>,
            <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Containments,
            <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Indices,
            <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::ReshapeIndices,>>::Canonical, <<R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Viewable as ContainsViewsOuter<'a, T::EntryViews<'a>,
            <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Containments,
            <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Indices,
            <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::ReshapeIndices,>>::Canonical, MergeContainments>>::Merged, I, P, R, RI>>::Decision,
            <ResourcesClaims as Claims<
                'a,
                T::ResourceViews<'a>,
                ResourcesIndices,
                ResourcesContainments,
                Resources,
                ResourcesInverseIndices,
            >>::Decision,
        ) as Merger>::Decision,
        (<<R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Viewable as view::Merge<<<R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Viewable as ContainsViewsOuter<'a, T::Views<'a>,
        <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Containments,
        <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Indices,
        <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::ReshapeIndices,>>::Canonical, <<R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Viewable as ContainsViewsOuter<'a, T::EntryViews<'a>,
        <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Containments,
        <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Indices,
        <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::ReshapeIndices,>>::Canonical, MergeContainments>>::Merged, C),
        IS,
        PS,
        RIS,
        MergeParameters,
        (T::ResourceViews<'a>, ResourcesClaims),
        ResourcesIndicesList,
        ResourcesContainmentsList,
        ResourcesInverseIndicesList,
        QueryIndicesList,
        ResourceViewsIndicesList,
        DisjointIndicesList,
        EntryIndicesList,
    >>::Stage;
    type Remainder = <(task::ParSystem<T>, U) as Cutoff<
        'a,
        R,
        Resources,
        <(
            <C as Claims<'a, <<R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Viewable as view::Merge<<<R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Viewable as ContainsViewsOuter<'a, T::Views<'a>,
            <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Containments,
            <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Indices,
            <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::ReshapeIndices,>>::Canonical, <<R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Viewable as ContainsViewsOuter<'a, T::EntryViews<'a>,
            <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Containments,
            <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Indices,
            <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::ReshapeIndices,>>::Canonical, MergeContainments>>::Merged, I, P, R, RI>>::Decision,
            <ResourcesClaims as Claims<
                'a,
                T::ResourceViews<'a>,
                ResourcesIndices,
                ResourcesContainments,
                Resources,
                ResourcesInverseIndices,
            >>::Decision,
        ) as Merger>::Decision,
        (<<R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Viewable as view::Merge<<<R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Viewable as ContainsViewsOuter<'a, T::Views<'a>,
        <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Containments,
        <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::Indices,
        <R as ContainsViewsSealed<'a, T::Views<'a>, LeftMergeIndices>>::ReshapeIndices,>>::Canonical, <<R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Viewable as ContainsViewsOuter<'a, T::EntryViews<'a>,
        <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Containments,
        <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::Indices,
        <R as ContainsViewsSealed<'a, T::EntryViews<'a>, RightMergeIndices>>::ReshapeIndices,>>::Canonical, MergeContainments>>::Merged, C),
        IS,
        PS,
        RIS,
        MergeParameters,
        (T::ResourceViews<'a>, ResourcesClaims),
        ResourcesIndicesList,
        ResourcesContainmentsList,
        ResourcesInverseIndicesList,
        QueryIndicesList,
        ResourceViewsIndicesList,
        DisjointIndicesList,
        EntryIndicesList,
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
    QueryIndicesList,
    ResourceViewsIndicesList,
    DisjointIndicesList,
    EntryIndicesList,
> where
    R: Registry,
{
    type Stage: Stage<
        'a,
        R,
        Resources,
        QueryIndicesList,
        ResourceViewsIndicesList,
        DisjointIndicesList,
        EntryIndicesList,
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
        QueryIndices,
        QueryIndicesList,
        ResourceViewsIndices,
        ResourceViewsIndicesList,
        DisjointIndices,
        DisjointIndicesList,
        EntryIndices,
        EntryIndicesList,
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
        (QueryIndices, QueryIndicesList),
        (ResourceViewsIndices, ResourceViewsIndicesList),
        (DisjointIndices, DisjointIndicesList),
        (EntryIndices, EntryIndicesList),
    > for (T, U)
where
    R: ContainsQuery<'a, T::Filter, T::Views, QueryIndices>,
    Resources: 'a,
    T: Task<'a, R, Resources, QueryIndices, ResourceViewsIndices, DisjointIndices, EntryIndices>
        + Send
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
        QueryIndicesList,
        ResourceViewsIndicesList,
        DisjointIndicesList,
        EntryIndicesList,
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
            QueryIndicesList,
            ResourceViewsIndicesList,
            DisjointIndicesList,
            EntryIndicesList,
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
        QueryIndicesList,
        ResourceViewsIndicesList,
        DisjointIndicesList,
        EntryIndicesList,
    >>::Remainder;

    #[inline]
    fn cutoff_stage(&'a mut self) -> (Self::Stage, &'a mut Self::Remainder) {
        let (stage, remainder) = self.1.extract_stage();
        ((&mut self.0, stage), remainder)
    }
}
