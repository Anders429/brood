use crate::{
    query::filter::Filter,
    registry::Registry,
    system::{
        schedule::{
            sendable::SendableWorld,
            stage::{Null, Stage},
        },
        ParSystem, System,
    },
    world::World,
};

pub trait Seal<'a, R, SFI, SVI, PFI, PVI>: Send
where
    R: Registry + 'a,
{
    fn run(&mut self, world: SendableWorld<R>);

    fn defer(&mut self, world: SendableWorld<R>);

    fn run_current(&mut self, world: SendableWorld<R>);

    fn run_continuing(&mut self, world: SendableWorld<R>);

    fn flush(&mut self, world: SendableWorld<R>);
}

impl<'a, R> Seal<'a, R, Null, Null, Null, Null> for Null
where
    R: Registry + 'a,
{
    fn run(&mut self, _world: SendableWorld<R>) {}

    fn defer(&mut self, _world: SendableWorld<R>) {}

    fn run_current(&mut self, _world: SendableWorld<R>) {}

    fn run_continuing(&mut self, _world: SendableWorld<R>) {}

    fn flush(&mut self, _world: SendableWorld<R>) {}
}

impl<'a, S, P, L, R, SFI, SFIS, SVI, SVIS, PFI, PFIS, PVI, PVIS>
    Seal<'a, R, (SFI, SFIS), (SVI, SVIS), (PFI, PFIS), (PVI, PVIS)> for (Stage<S, P>, L)
where
    R: Registry + 'a,
    S: System<'a> + Send,
    S::Filter: Filter<R, SFI>,
    S::Views: Filter<R, SVI>,
    P::Filter: Filter<R, PFI>,
    P::Views: Filter<R, PVI>,
    P: ParSystem<'a> + Send,
    L: Seal<'a, R, SFIS, SVIS, PFIS, PVIS>,
{
    fn run(&mut self, world: SendableWorld<R>) {
        self.defer(world);
        self.run_current(world);
    }

    fn defer(&mut self, world: SendableWorld<R>) {
        match self.0 {
            Stage::Start(_) | Stage::Flush => {
                self.1.run(world);
            }
            Stage::Continue(_) => {
                self.1.defer(world);
            }
        }
    }

    fn run_current(&mut self, world: SendableWorld<R>) {
        match &mut self.0 {
            Stage::Start(task) => {
                task.run(world);
            }
            Stage::Continue(task) => {
                rayon::join(
                    || {
                        self.1.run_continuing(world);
                    },
                    || {
                        task.run(world);
                    },
                );
            }
            Stage::Flush => {
                self.1.flush(world);
            }
        }
    }

    fn run_continuing(&mut self, world: SendableWorld<R>) {
        match &mut self.0 {
            Stage::Start(task) => {
                task.run(world);
            }
            Stage::Continue(task) => {
                rayon::join(
                    || {
                        self.1.run_continuing(world);
                    },
                    || {
                        task.run(world);
                    },
                );
            }
            Stage::Flush => {}
        }
    }

    fn flush(&mut self, world: SendableWorld<R>) {
        match &mut self.0 {
            Stage::Start(task) => {
                task.flush(
                    // SAFETY: This is guaranteed to be the only reference to this `World<R>`,
                    // meaning this cast to a mutable reference is sound.
                    unsafe { &mut *(world.get() as *const World<R> as *mut World<R>) },
                );
            }
            Stage::Continue(task) => {
                self.1.flush(world);
                task.flush(
                    // SAFETY: This is guaranteed to be the only reference to this `World<R>`,
                    // meaning this cast to a mutable reference is sound.
                    unsafe { &mut *(world.get() as *const World<R> as *mut World<R>) },
                );
            }
            Stage::Flush => {}
        }
    }
}
