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

pub trait CanonicalViews<'a, Views, Containments> {
    fn view(&'a mut self) -> Views;
}

impl<'a> CanonicalViews<'a, view::Null, contains::Null> for Null {
    fn view(&'a mut self) -> view::Null {
        view::Null
    }
}

impl<'a, Resource, Resources, Views, Containments>
    CanonicalViews<'a, (&'a Resource, Views), (Contained, Containments)> for (Resource, Resources)
where
    Resources: CanonicalViews<'a, Views, Containments>,
{
    fn view(&'a mut self) -> (&'a Resource, Views) {
        (&self.0, self.1.view())
    }
}

impl<'a, Resource, Resources, Views, Containments>
    CanonicalViews<'a, (&'a mut Resource, Views), (Contained, Containments)>
    for (Resource, Resources)
where
    Resources: CanonicalViews<'a, Views, Containments>,
{
    fn view(&'a mut self) -> (&'a mut Resource, Views) {
        (&mut self.0, self.1.view())
    }
}

impl<'a, Resource, Resources, Views, Containments>
    CanonicalViews<'a, Views, (NotContained, Containments)> for (Resource, Resources)
where
    Resources: CanonicalViews<'a, Views, Containments>,
{
    fn view(&'a mut self) -> Views {
        self.1.view()
    }
}
