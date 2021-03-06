use crate::{
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

pub trait Seal<'a>: Send {
    fn run<R>(&mut self, world: SendableWorld<R>)
    where
        R: Registry + 'a;

    fn defer<R>(&mut self, world: SendableWorld<R>)
    where
        R: Registry + 'a;

    fn run_current<R>(&mut self, world: SendableWorld<R>)
    where
        R: Registry + 'a;

    fn run_continuing<R>(&mut self, world: SendableWorld<R>)
    where
        R: Registry + 'a;

    fn flush<R>(&mut self, world: SendableWorld<R>)
    where
        R: Registry + 'a;
}

impl<'a> Seal<'a> for Null {
    fn run<R>(&mut self, _world: SendableWorld<R>)
    where
        R: Registry + 'a,
    {
    }

    fn defer<R>(&mut self, _world: SendableWorld<R>)
    where
        R: Registry + 'a,
    {
    }

    fn run_current<R>(&mut self, _world: SendableWorld<R>)
    where
        R: Registry + 'a,
    {
    }

    fn run_continuing<R>(&mut self, _world: SendableWorld<R>)
    where
        R: Registry + 'a,
    {
    }

    fn flush<R>(&mut self, _world: SendableWorld<R>)
    where
        R: Registry + 'a,
    {
    }
}

impl<'a, S, P, L> Seal<'a> for (Stage<S, P>, L)
where
    S: System<'a> + Send,
    P: ParSystem<'a> + Send,
    L: Seal<'a>,
{
    fn run<R>(&mut self, world: SendableWorld<R>)
    where
        R: Registry + 'a,
    {
        self.defer(world);
        self.run_current(world);
    }

    fn defer<R>(&mut self, world: SendableWorld<R>)
    where
        R: Registry + 'a,
    {
        match self.0 {
            Stage::Start(_) | Stage::Flush => {
                self.1.run(world);
            }
            Stage::Continue(_) => {
                self.1.defer(world);
            }
        }
    }

    fn run_current<R>(&mut self, world: SendableWorld<R>)
    where
        R: Registry + 'a,
    {
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

    fn run_continuing<R>(&mut self, world: SendableWorld<R>)
    where
        R: Registry + 'a,
    {
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

    fn flush<R>(&mut self, world: SendableWorld<R>)
    where
        R: Registry + 'a,
    {
        match &mut self.0 {
            Stage::Start(task) => {
                task.flush(
                    // SAFETY: This is guaranteed to be the only reference to this `World<R>`,
                    // meaning this cast to a mutable reference is sound.
                    unsafe { &mut *(world.0 as *const World<R> as *mut World<R>) },
                );
            }
            Stage::Continue(task) => {
                self.1.flush(world);
                task.flush(
                    // SAFETY: This is guaranteed to be the only reference to this `World<R>`,
                    // meaning this cast to a mutable reference is sound.
                    unsafe { &mut *(world.0 as *const World<R> as *mut World<R>) },
                );
            }
            Stage::Flush => {}
        }
    }
}
