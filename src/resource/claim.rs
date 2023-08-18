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

    fn empty_claims() -> Self::Claims;
}

impl Claims for Null {
    type Claims = claim::Null;

    fn empty_claims() -> Self::Claims {
        claim::Null
    }
}

impl<Resource, Resources> Claims for (Resource, Resources)
where
    Resources: Claims,
{
    type Claims = (Claim, Resources::Claims);

    fn empty_claims() -> Self::Claims {
        (Claim::None, Resources::empty_claims())
    }
}
