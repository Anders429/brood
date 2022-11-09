use crate::{
    query::claim::Claim,
    system::{
        schedule::{
            raw_task::{
                Null,
                RawTask,
            },
            stage,
            stage::Stage,
        },
        ParSystem,
        System,
    },
};
use core::any::TypeId;
use fnv::FnvBuildHasher;
use hashbrown::HashSet;

pub trait Sealed {
    type Stages;

    fn into_stages(
        self,
        mutable_claims: &mut HashSet<TypeId, FnvBuildHasher>,
        immutable_claims: &mut HashSet<TypeId, FnvBuildHasher>,
        mutable_buffer: &mut HashSet<TypeId, FnvBuildHasher>,
        immutable_buffer: &mut HashSet<TypeId, FnvBuildHasher>,
    ) -> Self::Stages;
}

impl Sealed for Null {
    type Stages = stage::Null;

    fn into_stages(
        self,
        _mutable_claims: &mut HashSet<TypeId, FnvBuildHasher>,
        _immutable_claims: &mut HashSet<TypeId, FnvBuildHasher>,
        _mutable_buffer: &mut HashSet<TypeId, FnvBuildHasher>,
        _immutable_buffer: &mut HashSet<TypeId, FnvBuildHasher>,
    ) -> Self::Stages {
        stage::Null
    }
}

impl<S, P, T> Sealed for (RawTask<S, P>, T)
where
    S: System + Send,
    P: ParSystem + Send,
    T: Sealed,
{
    type Stages = (Stage<S, P>, T::Stages);

    fn into_stages(
        self,
        mutable_claims: &mut HashSet<TypeId, FnvBuildHasher>,
        immutable_claims: &mut HashSet<TypeId, FnvBuildHasher>,
        mutable_buffer: &mut HashSet<TypeId, FnvBuildHasher>,
        immutable_buffer: &mut HashSet<TypeId, FnvBuildHasher>,
    ) -> Self::Stages {
        let prev_stages = self.1.into_stages(
            mutable_claims,
            immutable_claims,
            mutable_buffer,
            immutable_buffer,
        );

        match self.0 {
            RawTask::Task(task) => {
                // Helper function to check whether the intersection betwen two sets is nonempty.
                fn intersects(
                    a: &HashSet<TypeId, FnvBuildHasher>,
                    b: &HashSet<TypeId, FnvBuildHasher>,
                ) -> bool {
                    a.intersection(b).next().is_some()
                }

                mutable_buffer.clear();
                immutable_buffer.clear();

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
