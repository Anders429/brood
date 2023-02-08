use crate::resource::Null;

pub trait Length {
    const LEN: usize;
}

impl Length for Null {
    const LEN: usize = 0;
}

impl<Resource, Resources> Length for (Resource, Resources)
where
    Resources: Length,
{
    const LEN: usize = Resources::LEN + 1;
}
