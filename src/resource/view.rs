#[cfg(feature = "rayon")]
use crate::query::view::{
    claim,
    claim::Claim,
};
use crate::{
    query::view,
    resource,
    resource::{
        contains,
        contains::{
            Contained,
            NotContained,
        },
        Null,
    },
};

pub trait CanonicalViews<'a, Views, Containments>: resource::Resources {
    fn view(&'a mut self) -> Views;

    /// Return the dynamic claims over the resources borrowed by the `Views`.
    #[cfg(feature = "rayon")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
    fn claims() -> Self::Claims;
}

impl<'a> CanonicalViews<'a, view::Null, contains::Null> for Null {
    fn view(&'a mut self) -> view::Null {
        view::Null
    }

    #[cfg(feature = "rayon")]
    fn claims() -> Self::Claims {
        claim::Null
    }
}

impl<'a, Resource, Resources, Views, Containments>
    CanonicalViews<'a, (&'a Resource, Views), (Contained, Containments)> for (Resource, Resources)
where
    Resource: resource::Resource,
    Resources: CanonicalViews<'a, Views, Containments>,
{
    fn view(&'a mut self) -> (&'a Resource, Views) {
        (&self.0, self.1.view())
    }

    #[cfg(feature = "rayon")]
    fn claims() -> Self::Claims {
        (Claim::Immutable, Resources::claims())
    }
}

impl<'a, Resource, Resources, Views, Containments>
    CanonicalViews<'a, (&'a mut Resource, Views), (Contained, Containments)>
    for (Resource, Resources)
where
    Resource: resource::Resource,
    Resources: CanonicalViews<'a, Views, Containments>,
{
    fn view(&'a mut self) -> (&'a mut Resource, Views) {
        (&mut self.0, self.1.view())
    }

    #[cfg(feature = "rayon")]
    fn claims() -> Self::Claims {
        (Claim::Mutable, Resources::claims())
    }
}

impl<'a, Resource, Resources, Views, Containments>
    CanonicalViews<'a, Views, (NotContained, Containments)> for (Resource, Resources)
where
    Resource: resource::Resource,
    Resources: CanonicalViews<'a, Views, Containments>,
{
    fn view(&'a mut self) -> Views {
        self.1.view()
    }

    #[cfg(feature = "rayon")]
    fn claims() -> Self::Claims {
        (Claim::None, Resources::claims())
    }
}
