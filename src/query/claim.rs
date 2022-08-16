use crate::{component::Component, entity, query::view};
use core::any::TypeId;
use hashbrown::HashSet;

pub trait Claim {
    fn claim(mutable_claims: &mut HashSet<TypeId, ahash::RandomState>, immutable_claims: &mut HashSet<TypeId, ahash::RandomState>);
}

impl<C> Claim for &C
where
    C: Component,
{
    fn claim(_mutable_claims: &mut HashSet<TypeId, ahash::RandomState>, immutable_claims: &mut HashSet<TypeId, ahash::RandomState>) {
        immutable_claims.insert(TypeId::of::<C>());
    }
}

impl<C> Claim for &mut C
where
    C: Component,
{
    fn claim(mutable_claims: &mut HashSet<TypeId, ahash::RandomState>, _immutable_claims: &mut HashSet<TypeId, ahash::RandomState>) {
        mutable_claims.insert(TypeId::of::<C>());
    }
}

impl<C> Claim for Option<&C>
where
    C: Component,
{
    fn claim(_mutable_claims: &mut HashSet<TypeId, ahash::RandomState>, immutable_claims: &mut HashSet<TypeId, ahash::RandomState>) {
        immutable_claims.insert(TypeId::of::<C>());
    }
}

impl<C> Claim for Option<&mut C>
where
    C: Component,
{
    fn claim(mutable_claims: &mut HashSet<TypeId, ahash::RandomState>, _immutable_claims: &mut HashSet<TypeId, ahash::RandomState>) {
        mutable_claims.insert(TypeId::of::<C>());
    }
}

impl Claim for entity::Identifier {
    fn claim(_mutable_claims: &mut HashSet<TypeId, ahash::RandomState>, _immutable_claims: &mut HashSet<TypeId, ahash::RandomState>) {}
}

impl Claim for view::Null {
    fn claim(_mutable_claims: &mut HashSet<TypeId, ahash::RandomState>, _immutable_claims: &mut HashSet<TypeId, ahash::RandomState>) {}
}

impl<V, W> Claim for (V, W)
where
    V: Claim,
    W: Claim,
{
    fn claim(mutable_claims: &mut HashSet<TypeId, ahash::RandomState>, immutable_claims: &mut HashSet<TypeId, ahash::RandomState>) {
        V::claim(mutable_claims, immutable_claims);
        W::claim(mutable_claims, immutable_claims);
    }
}
