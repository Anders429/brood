use crate::{
    query::{claim::Claim, view, view::seal::ViewsSeal},
    system::{
        schedule::{
            raw_task::{Null, RawTask},
            stage,
            stage::{Stage, Stages},
        },
        ParSystem, System,
    },
};
use core::any::TypeId;
use hashbrown::HashSet;

pub trait Seal<'a> {
    type Stages: Stages<'a>;

    fn into_stages(
        self,
        mutable_claims: &mut HashSet<TypeId, ahash::RandomState>,
        immutable_claims: &mut HashSet<TypeId, ahash::RandomState>,
        mutable_buffer: &mut HashSet<TypeId, ahash::RandomState>,
        immutable_buffer: &mut HashSet<TypeId, ahash::RandomState>,
        view_assertion_buffer: &mut view::AssertionBuffer,
    ) -> Self::Stages;
}

impl<'a> Seal<'a> for Null {
    type Stages = stage::Null;

    fn into_stages(
        self,
        _mutable_claims: &mut HashSet<TypeId, ahash::RandomState>,
        _immutable_claims: &mut HashSet<TypeId, ahash::RandomState>,
        _mutable_buffer: &mut HashSet<TypeId, ahash::RandomState>,
        _immutable_buffer: &mut HashSet<TypeId, ahash::RandomState>,
        _view_assertion_buffer: &mut view::AssertionBuffer,
    ) -> Self::Stages {
        stage::Null
    }
}

impl<'a, S, P, T> Seal<'a> for (RawTask<S, P>, T)
where
    S: System<'a> + Send,
    P: ParSystem<'a> + Send,
    T: Seal<'a>,
{
    type Stages = (Stage<S, P>, T::Stages);

    fn into_stages(
        self,
        mutable_claims: &mut HashSet<TypeId, ahash::RandomState>,
        immutable_claims: &mut HashSet<TypeId, ahash::RandomState>,
        mutable_buffer: &mut HashSet<TypeId, ahash::RandomState>,
        immutable_buffer: &mut HashSet<TypeId, ahash::RandomState>,
        view_assertion_buffer: &mut view::AssertionBuffer,
    ) -> Self::Stages {
        let prev_stages = self.1.into_stages(
            mutable_claims,
            immutable_claims,
            mutable_buffer,
            immutable_buffer,
            view_assertion_buffer,
        );

        match self.0 {
            RawTask::Task(task) => {
                // Helper function to check whether the intersection betwen two sets is nonempty.
                fn intersects(a: &HashSet<TypeId, ahash::RandomState>, b: &HashSet<TypeId, ahash::RandomState>) -> bool {
                    a.intersection(b).next().is_some()
                }

                mutable_buffer.clear();
                immutable_buffer.clear();

                // Assert that this system's views are sound.
                view_assertion_buffer.clear();
                S::Views::assert_claims(view_assertion_buffer);
                view_assertion_buffer.clear();
                P::Views::assert_claims(view_assertion_buffer);

                // Identify this stage's claims on components.
                S::Views::claim(mutable_buffer, immutable_buffer);
                P::Views::claim(mutable_buffer, immutable_buffer);

                // If the claims are incompatible, a new stage must begin.
                //
                // Claims are incompatible if an immutable claim is made on a component already
                // mutable claimed, or if a mutable claim is made on a component already claimed at
                // all.
                if intersects(immutable_buffer, mutable_claims)
                    || intersects(mutable_buffer, mutable_claims)
                    || intersects(mutable_buffer, immutable_claims)
                {
                    (Stage::Start(task), prev_stages)
                } else {
                    (Stage::Continue(task), prev_stages)
                }
            }
            RawTask::Flush => {
                mutable_claims.clear();
                immutable_claims.clear();
                (Stage::Flush, prev_stages)
            }
        }
    }
}
