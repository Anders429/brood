mod index {
    pub(super) enum Index {}
}

pub trait Get<Resource, Index> {
    type View;
    type Remainder;

    fn get(self) -> (Self::View, Self::Remainder);
}

impl<'a, Resource, Resources> Get<Resource, index::Index> for (&'a Resource, Resources) {
    type View = &'a Resource;
    type Remainder = Resources;

    fn get(self) -> (Self::View, Self::Remainder) {
        self
    }
}

impl<'a, Resource, Resources> Get<Resource, index::Index>
    for (&'a mut Resource, Resources)
{
    type View = &'a mut Resource;
    type Remainder = Resources;

    fn get(self) -> (Self::View, Self::Remainder) {
        self
    }
}

impl<View, CurrentView, Views, Index> Get<View, (Index,)> for (CurrentView, Views)
where
    Views: Get<View, Index>,
{
    type View = Views::View;
    type Remainder = (CurrentView, Views::Remainder);

    fn get(self) -> (Self::View, Self::Remainder) {
        let (view, remainder) = self.1.get();
        (view, (self.0, remainder))
    }
}
