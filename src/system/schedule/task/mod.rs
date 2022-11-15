mod sealed;

pub(crate) use sealed::Task;

use crate::hlist::define_null;

define_null!();

pub struct System<S>(pub S);

pub struct ParSystem<P>(pub P);
