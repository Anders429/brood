use crate::{query::view::{resource::Get, Null}, resource::contains};

pub trait Reshape<ReshapedViews, Indices> {
    fn reshape(self) -> ReshapedViews;
}

impl Reshape<Null, contains::Null> for Null {
    fn reshape(self) -> Null {
        Null
    }
}

impl<View, Views, ReshapedViews, Index, Indices> Reshape<(View, ReshapedViews), (Index, Indices)> for Views
where
    Views: Get<View, Index, View = View>,
    Views::Remainder: Reshape<ReshapedViews, Indices>,
{
    fn reshape(self) -> (View, ReshapedViews) {
        let (view, remainder) = self.get();
        (view, remainder.reshape())
    }
}
