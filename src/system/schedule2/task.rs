use crate::{hlist::define_null, query::Query, registry::{Registry, ContainsQuery, ContainsParQuery}, system, system::{schedule2::sendable::SendableWorld}};

define_null!();

pub trait Task<'a, R, SFI, SVI, SP, SI, SQ> where R: Registry {
    fn run(&mut self, world: SendableWorld<R>);
}

pub struct System<S>(pub S);

impl<'a, R, S, SFI, SVI, SP, SI, SQ> Task<'a, R, SFI, SVI, SP, SI, SQ> for System<S> where S: system::System + Send, R: ContainsQuery<'a, S::Filter, SFI, S::Views<'a>, SVI, SP, SI, SQ> {
    fn run(&mut self, world: SendableWorld<R>) {
        // Query world using system.
        let result =
            // SAFETY: The access to the world's components follows Rust's borrowing rules.
            unsafe { (*world.get()).query(Query::<S::Views<'a>, S::Filter>::new()) };
        // Run system using the query result.
        self.0.run(result);
    }
}

pub struct ParSystem<P>(pub P);

impl<'a, P, R, SFI, SVI, SP, SI, SQ> Task<'a, R, SFI, SVI, SP, SI, SQ> for ParSystem<P> where P: system::ParSystem + Send, R: ContainsParQuery<'a, P::Filter, SFI, P::Views<'a>, SVI, SP, SI, SQ> {
    fn run(&mut self, world: SendableWorld<R>) {
        // Query world using system.
        let result =
            // SAFETY: The access to the world's components follows Rust's borrowing rules.
            unsafe { (*world.get()).par_query(Query::<P::Views<'a>, P::Filter>::new()) };
        // Run system using the query result.
        self.0.run(result);
    }
}
