use crate::{component::Component, entity, query::view};
use core::any::TypeId;
use hashbrown::HashSet;

pub trait Claim {
    fn claim(mutable_claims: &mut HashSet<TypeId>, immutable_claims: &mut HashSet<TypeId>);
}

impl<C> Claim for &C
where
    C: Component,
{
    fn claim(_mutable_claims: &mut HashSet<TypeId>, immutable_claims: &mut HashSet<TypeId>) {
        immutable_claims.insert(TypeId::of::<C>());
    }
}

impl<C> Claim for &mut C
where
    C: Component,
{
    fn claim(mutable_claims: &mut HashSet<TypeId>, _immutable_claims: &mut HashSet<TypeId>) {
        mutable_claims.insert(TypeId::of::<C>());
    }
}

impl<C> Claim for Option<&C>
where
    C: Component,
{
    fn claim(_mutable_claims: &mut HashSet<TypeId>, immutable_claims: &mut HashSet<TypeId>) {
        immutable_claims.insert(TypeId::of::<C>());
    }
}

impl<C> Claim for Option<&mut C>
where
    C: Component,
{
    fn claim(mutable_claims: &mut HashSet<TypeId>, _immutable_claims: &mut HashSet<TypeId>) {
        mutable_claims.insert(TypeId::of::<C>());
    }
}

impl Claim for entity::Identifier {
    fn claim(_mutable_claims: &mut HashSet<TypeId>, _immutable_claims: &mut HashSet<TypeId>) {}
}

impl Claim for view::Null {
    fn claim(_mutable_claims: &mut HashSet<TypeId>, _immutable_claims: &mut HashSet<TypeId>) {}
}

impl<'a, V, W> Claim for (V, W)
where
    V: Claim,
    W: Claim,
{
    fn claim(mutable_claims: &mut HashSet<TypeId>, immutable_claims: &mut HashSet<TypeId>) {
        V::claim(mutable_claims, immutable_claims);
        W::claim(mutable_claims, immutable_claims);
    }
}