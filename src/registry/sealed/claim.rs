use crate::{
    component::Component,
    query::{
        view,
        view::{
            claim,
            Claim,
        },
    },
    registry::Null,
};
use core::{
    fmt::Debug,
    hash::Hash,
};

pub trait Claims {
    type Claims: view::Claims + Clone + Debug + Eq + Hash + Send;

    fn empty_claims() -> Self::Claims;
}

impl Claims for Null {
    type Claims = claim::Null;

    fn empty_claims() -> Self::Claims {
        claim::Null
    }
}

impl<C, R> Claims for (C, R)
where
    C: Component,
    R: Claims,
{
    type Claims = (Claim, R::Claims);

    fn empty_claims() -> Self::Claims {
        (Claim::None, R::empty_claims())
    }
}
