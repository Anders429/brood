use crate::{
    query::{
        view,
        view::{
            claim,
            Claim,
        },
    },
    resource::Null,
};
use core::{
    fmt::Debug,
    hash::Hash,
};

pub trait Claims {
    type Claims: view::Claims + Clone + Debug + Eq + Hash + Send + Default;
}

impl Claims for Null {
    type Claims = claim::Null;
}

impl<Resource, Resources> Claims for (Resource, Resources)
where
    Resources: Claims,
{
    type Claims = (Claim, Resources::Claims);
}
