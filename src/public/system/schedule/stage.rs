use crate::{
    internal::system::schedule::stage::StagesSeal,
    system::{schedule::task::Task, ParSystem, System},
};

pub enum Stage<S, P> {
    Start(Task<S, P>),
    Continue(Task<S, P>),
    Flush,
}

pub trait Stages<'a>: StagesSeal<'a> {}

pub struct Null;

impl<'a> Stages<'a> for Null {}

impl<'a, S, P, L> Stages<'a> for (Stage<S, P>, L)
where
    S: System<'a> + Send,
    P: ParSystem<'a> + Send,
    L: Stages<'a>,
{}
