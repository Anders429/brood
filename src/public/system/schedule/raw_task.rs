use crate::{
    internal::system::schedule::raw_task::RawTasksSeal,
    system::{schedule::task::Task, ParSystem, System},
};

pub enum RawTask<S, P> {
    Task(Task<S, P>),
    Flush,
}

pub struct Null;

pub trait RawTasks<'a>: RawTasksSeal<'a> {}

impl<'a> RawTasks<'a> for Null {}

impl<'a, S, P, T> RawTasks<'a> for (RawTask<S, P>, T)
where
    S: System<'a> + Send,
    P: ParSystem<'a> + Send,
    T: RawTasks<'a>,
{
}
