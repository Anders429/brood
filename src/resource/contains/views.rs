//! Given a list of resource views, and a list of resources (not guaranteed to be in the same
//! order), we return the borrowed resources as specified by the views.
#[cfg(feature = "rayon")]
use crate::query::view::{
    claim,
    claim::Claim,
};
use crate::{
    query::{
        view,
        view::resource::Reshape,
    },
    resource,
    resource::{
        contains::{
            Contained,
            NotContained,
            Null,
        },
        view::CanonicalViews,
    },
};

/// Indicates that all of the viewed resources are contained in the list of resources.
pub trait ContainsViews<'a, Views, Indices>: Sealed<'a, Views, Indices> {}

impl<'a, Resources, Views, Indices> ContainsViews<'a, Views, Indices> for Resources where
    Resources: Sealed<'a, Views, Indices>
{
}

pub trait Sealed<'a, Views, Indices>: resource::Resources {
    fn view(&'a mut self) -> Views;

    #[cfg(feature = "rayon")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
    fn claims() -> Self::Claims;
}

impl<'a, Resources, Views, Containments, Indices, CanonicalContainments, ReshapeIndices>
    Sealed<'a, Views, (Containments, Indices, CanonicalContainments, ReshapeIndices)> for Resources
where
    Resources: Expanded<'a, Views, Containments, Indices, CanonicalContainments, ReshapeIndices>,
{
    fn view(&'a mut self) -> Views {
        self.view()
    }

    #[cfg(feature = "rayon")]
    fn claims() -> Self::Claims {
        Self::claims()
    }
}

pub trait Expanded<'a, Views, Containments, Indices, CanonicalContainments, ReshapeIndices>:
    resource::Resources
{
    /// The canonical form of the `Views` with respect to the resources.
    type Canonical: Reshape<Views, ReshapeIndices>;

    fn view(&'a mut self) -> Views;

    #[cfg(feature = "rayon")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
    fn claims() -> Self::Claims;
}

impl<'a, ReshapeIndices> Expanded<'a, view::Null, Null, Null, Null, ReshapeIndices>
    for resource::Null
where
    view::Null: Reshape<view::Null, ReshapeIndices>,
{
    type Canonical = view::Null;

    fn view(&'a mut self) -> view::Null {
        view::Null
    }

    #[cfg(feature = "rayon")]
    fn claims() -> Self::Claims {
        claim::Null
    }
}

impl<
        'a,
        Resource,
        Resources,
        Views,
        Containments,
        Indices,
        CanonicalContainments,
        ReshapeIndices,
    >
    Expanded<
        'a,
        Views,
        (NotContained, Containments),
        Indices,
        (NotContained, CanonicalContainments),
        ReshapeIndices,
    > for (Resource, Resources)
where
    Resource: resource::Resource,
    Resources: Expanded<'a, Views, Containments, Indices, CanonicalContainments, ReshapeIndices>,
{
    type Canonical = Resources::Canonical;

    fn view(&'a mut self) -> Views {
        self.1.view()
    }

    #[cfg(feature = "rayon")]
    fn claims() -> Self::Claims {
        (Claim::None, Resources::claims())
    }
}

impl<
        'a,
        Resource,
        Resources,
        Views,
        Containments,
        Index,
        Indices,
        CanonicalContainment,
        CanonicalContainments,
        ReshapeIndex,
        ReshapeIndices,
    >
    Expanded<
        'a,
        Views,
        (Contained, Containments),
        (Index, Indices),
        (CanonicalContainment, CanonicalContainments),
        (ReshapeIndex, ReshapeIndices),
    > for (Resource, Resources)
where
    Views: view::resource::Get<Resource, Index>,
    Resources: Expanded<
        'a,
        Views::Remainder,
        Containments,
        Indices,
        CanonicalContainments,
        ReshapeIndices,
    >,
    (Resource, Resources): CanonicalViews<
        'a,
        (Views::View, Resources::Canonical),
        (CanonicalContainment, CanonicalContainments),
    >,
    (Views::View, Resources::Canonical): Reshape<Views, (ReshapeIndex, ReshapeIndices)>,
{
    type Canonical = (Views::View, Resources::Canonical);

    fn view(&'a mut self) -> Views {
        CanonicalViews::view(self).reshape()
    }

    #[cfg(feature = "rayon")]
    fn claims() -> Self::Claims {
        <Self as CanonicalViews<
            'a,
            (Views::View, Resources::Canonical),
            (CanonicalContainment, CanonicalContainments),
        >>::claims()
    }
}
