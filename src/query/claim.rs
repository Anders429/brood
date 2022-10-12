use crate::{
    component::Component,
    entity,
    query::view,
};
use core::any::TypeId;
use fnv::FnvBuildHasher;
use hashbrown::HashSet;

pub trait Claim {
    fn claim(
        mutable_claims: &mut HashSet<TypeId, FnvBuildHasher>,
        immutable_claims: &mut HashSet<TypeId, FnvBuildHasher>,
    );
}

impl<C> Claim for &C
where
    C: Component,
{
    fn claim(
        _mutable_claims: &mut HashSet<TypeId, FnvBuildHasher>,
        immutable_claims: &mut HashSet<TypeId, FnvBuildHasher>,
    ) {
        immutable_claims.insert(TypeId::of::<C>());
    }
}

impl<C> Claim for &mut C
where
    C: Component,
{
    fn claim(
        mutable_claims: &mut HashSet<TypeId, FnvBuildHasher>,
        _immutable_claims: &mut HashSet<TypeId, FnvBuildHasher>,
    ) {
        mutable_claims.insert(TypeId::of::<C>());
    }
}

impl<C> Claim for Option<&C>
where
    C: Component,
{
    fn claim(
        _mutable_claims: &mut HashSet<TypeId, FnvBuildHasher>,
        immutable_claims: &mut HashSet<TypeId, FnvBuildHasher>,
    ) {
        immutable_claims.insert(TypeId::of::<C>());
    }
}

impl<C> Claim for Option<&mut C>
where
    C: Component,
{
    fn claim(
        mutable_claims: &mut HashSet<TypeId, FnvBuildHasher>,
        _immutable_claims: &mut HashSet<TypeId, FnvBuildHasher>,
    ) {
        mutable_claims.insert(TypeId::of::<C>());
    }
}

impl Claim for entity::Identifier {
    fn claim(
        _mutable_claims: &mut HashSet<TypeId, FnvBuildHasher>,
        _immutable_claims: &mut HashSet<TypeId, FnvBuildHasher>,
    ) {
    }
}

impl Claim for view::Null {
    fn claim(
        _mutable_claims: &mut HashSet<TypeId, FnvBuildHasher>,
        _immutable_claims: &mut HashSet<TypeId, FnvBuildHasher>,
    ) {
    }
}

impl<V, W> Claim for (V, W)
where
    V: Claim,
    W: Claim,
{
    fn claim(
        mutable_claims: &mut HashSet<TypeId, FnvBuildHasher>,
        immutable_claims: &mut HashSet<TypeId, FnvBuildHasher>,
    ) {
        V::claim(mutable_claims, immutable_claims);
        W::claim(mutable_claims, immutable_claims);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hashbrown::HashSet;

    struct A;
    struct B;

    #[test]
    fn claim_ref() {
        let mut mutable_claims = HashSet::with_hasher(FnvBuildHasher::default());
        let mut immutable_claims = HashSet::with_hasher(FnvBuildHasher::default());

        <&A>::claim(&mut mutable_claims, &mut immutable_claims);

        let expected_mutable_claims = HashSet::with_hasher(FnvBuildHasher::default());
        let mut expected_immutable_claims = HashSet::with_hasher(FnvBuildHasher::default());
        expected_immutable_claims.insert(TypeId::of::<A>());
        assert_eq!(mutable_claims, expected_mutable_claims);
        assert_eq!(immutable_claims, expected_immutable_claims);
    }

    #[test]
    fn claim_mut_ref() {
        let mut mutable_claims = HashSet::with_hasher(FnvBuildHasher::default());
        let mut immutable_claims = HashSet::with_hasher(FnvBuildHasher::default());

        <&mut A>::claim(&mut mutable_claims, &mut immutable_claims);

        let mut expected_mutable_claims = HashSet::with_hasher(FnvBuildHasher::default());
        let expected_immutable_claims = HashSet::with_hasher(FnvBuildHasher::default());
        expected_mutable_claims.insert(TypeId::of::<A>());
        assert_eq!(mutable_claims, expected_mutable_claims);
        assert_eq!(immutable_claims, expected_immutable_claims);
    }

    #[test]
    fn claim_option() {
        let mut mutable_claims = HashSet::with_hasher(FnvBuildHasher::default());
        let mut immutable_claims = HashSet::with_hasher(FnvBuildHasher::default());

        <Option<&A>>::claim(&mut mutable_claims, &mut immutable_claims);

        let expected_mutable_claims = HashSet::with_hasher(FnvBuildHasher::default());
        let mut expected_immutable_claims = HashSet::with_hasher(FnvBuildHasher::default());
        expected_immutable_claims.insert(TypeId::of::<A>());
        assert_eq!(mutable_claims, expected_mutable_claims);
        assert_eq!(immutable_claims, expected_immutable_claims);
    }

    #[test]
    fn claim_mut_option() {
        let mut mutable_claims = HashSet::with_hasher(FnvBuildHasher::default());
        let mut immutable_claims = HashSet::with_hasher(FnvBuildHasher::default());

        <Option<&mut A>>::claim(&mut mutable_claims, &mut immutable_claims);

        let mut expected_mutable_claims = HashSet::with_hasher(FnvBuildHasher::default());
        let expected_immutable_claims = HashSet::with_hasher(FnvBuildHasher::default());
        expected_mutable_claims.insert(TypeId::of::<A>());
        assert_eq!(mutable_claims, expected_mutable_claims);
        assert_eq!(immutable_claims, expected_immutable_claims);
    }

    #[test]
    fn claim_entity_identifier() {
        let mut mutable_claims = HashSet::with_hasher(FnvBuildHasher::default());
        let mut immutable_claims = HashSet::with_hasher(FnvBuildHasher::default());

        <entity::Identifier>::claim(&mut mutable_claims, &mut immutable_claims);

        let expected_mutable_claims = HashSet::with_hasher(FnvBuildHasher::default());
        let expected_immutable_claims = HashSet::with_hasher(FnvBuildHasher::default());
        assert_eq!(mutable_claims, expected_mutable_claims);
        assert_eq!(immutable_claims, expected_immutable_claims);
    }

    #[test]
    fn claim_view_null() {
        let mut mutable_claims = HashSet::with_hasher(FnvBuildHasher::default());
        let mut immutable_claims = HashSet::with_hasher(FnvBuildHasher::default());

        <view::Null>::claim(&mut mutable_claims, &mut immutable_claims);

        let expected_mutable_claims = HashSet::with_hasher(FnvBuildHasher::default());
        let expected_immutable_claims = HashSet::with_hasher(FnvBuildHasher::default());
        assert_eq!(mutable_claims, expected_mutable_claims);
        assert_eq!(immutable_claims, expected_immutable_claims);
    }

    #[test]
    fn claim_hlist() {
        let mut mutable_claims = HashSet::with_hasher(FnvBuildHasher::default());
        let mut immutable_claims = HashSet::with_hasher(FnvBuildHasher::default());

        <(&A, (&mut B, view::Null))>::claim(&mut mutable_claims, &mut immutable_claims);

        let mut expected_mutable_claims = HashSet::with_hasher(FnvBuildHasher::default());
        let mut expected_immutable_claims = HashSet::with_hasher(FnvBuildHasher::default());
        expected_immutable_claims.insert(TypeId::of::<A>());
        expected_mutable_claims.insert(TypeId::of::<B>());
        assert_eq!(mutable_claims, expected_mutable_claims);
        assert_eq!(immutable_claims, expected_immutable_claims);
    }
}
