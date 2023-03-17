use crate::{
    query::view::{
        resource::Get,
        Null,
    },
    resource::contains,
};

pub trait Reshape<ReshapedViews, Indices> {
    fn reshape(self) -> ReshapedViews;
}

impl Reshape<Null, contains::Null> for Null {
    fn reshape(self) -> Null {
        Null
    }
}

impl<'a, Resource, Views, ReshapedViews, Index, Indices> Reshape<(&'a Resource, ReshapedViews), (Index, Indices)>
    for Views
where
    Views: Get<Resource, Index, View = &'a Resource>,
    Views::Remainder: Reshape<ReshapedViews, Indices>,
{
    fn reshape(self) -> (&'a Resource, ReshapedViews) {
        let (view, remainder) = self.get();
        (view, remainder.reshape())
    }
}

impl<'a, Resource, Views, ReshapedViews, Index, Indices> Reshape<(&'a mut Resource, ReshapedViews), (Index, Indices)>
    for Views
where
    Views: Get<Resource, Index, View = &'a mut Resource>,
    Views::Remainder: Reshape<ReshapedViews, Indices>,
{
    fn reshape(self) -> (&'a mut Resource, ReshapedViews) {
        let (view, remainder) = self.get();
        (view, remainder.reshape())
    }
}
