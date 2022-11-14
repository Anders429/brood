use crate::hlist::define_null;

define_null!();

pub struct System<S>(S);

pub struct ParSystem<P>(P);
