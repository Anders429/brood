/// Given a list of resource views, and a list of resources (not guaranteed to be in the same
/// order), we return the borrowed resources as specified by the views.
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
pub trait ContainsViews<'a, Views, Containments, Indices, CanonicalContainments, ReshapeIndices>:
    Sealed<'a, Views, Containments, Indices, CanonicalContainments, ReshapeIndices>
{
}

impl<'a, Resources, Views, Containments, Indices, CanonicalContainments, ReshapeIndices>
    ContainsViews<'a, Views, Containments, Indices, CanonicalContainments, ReshapeIndices>
    for Resources
where
    Resources: Sealed<'a, Views, Containments, Indices, CanonicalContainments, ReshapeIndices>,
{
}

pub trait Sealed<'a, Views, Containments, Indices, CanonicalContainments, ReshapeIndices> {
    type Canonical: Reshape<Views, ReshapeIndices>;

    fn view(&'a mut self) -> Views;
}

impl<'a, ReshapeIndices> Sealed<'a, view::Null, Null, Null, Null, ReshapeIndices> for resource::Null
where
    view::Null: Reshape<view::Null, ReshapeIndices>,
{
    type Canonical = view::Null;

    fn view(&'a mut self) -> view::Null {
        view::Null
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
    Sealed<'a, Views, (NotContained, Containments), Indices, CanonicalContainments, ReshapeIndices>
    for (Resource, Resources)
where
    Resources: Sealed<'a, Views, Containments, Indices, CanonicalContainments, ReshapeIndices>,
{
    type Canonical = Resources::Canonical;

    fn view(&'a mut self) -> Views {
        self.1.view()
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
    Sealed<
        'a,
        Views,
        (Contained, Containments),
        (Index, Indices),
        (CanonicalContainment, CanonicalContainments),
        (ReshapeIndex, ReshapeIndices),
    > for (Resource, Resources)
where
    Views: view::resource::Get<Resource, Index>,
    Resources:
        Sealed<'a, Views::Remainder, Containments, Indices, CanonicalContainments, ReshapeIndices>,
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
}
