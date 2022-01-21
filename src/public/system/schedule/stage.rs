use crate::{
    registry::Registry,
    system::{schedule::{sendable::SendableWorld, task::Task}, ParSystem, System},
};

pub enum Stage<S, P> {
    Start(Task<S, P>),
    Continue(Task<S, P>),
    Flush,
}

// run -> runs the stages, starting with the innermost.
// defer -> skips the current stage, running the next-most inner stage.
// run_current_stage -> runs all tasks of the current stage.

pub trait Stages<'a>: Send {
    fn run<R>(&mut self, world: SendableWorld<'a, R>)
    where
        R: Registry + 'a;

    fn defer<R>(&mut self, world: SendableWorld<'a, R>)
    where
        R: Registry + 'a;

    fn run_current<R>(&mut self, world: SendableWorld<'a, R>)
    where
        R: Registry + 'a;

    fn run_continuing<R>(&mut self, world: SendableWorld<'a, R>)
    where
        R: Registry + 'a;

    fn flush<R>(&mut self, world: SendableWorld<'a, R>)
    where
        R: Registry + 'a;
}

pub struct Null;

impl<'a> Stages<'a> for Null {
    fn run<R>(&mut self, _world: SendableWorld<'a, R>)
    where
        R: Registry + 'a,
    {
    }

    fn defer<R>(&mut self, _world: SendableWorld<'a, R>)
    where
        R: Registry + 'a,
    {
    }

    fn run_current<R>(&mut self, _world: SendableWorld<'a, R>)
    where
        R: Registry + 'a,
    {
    }

    fn run_continuing<R>(&mut self, _world: SendableWorld<'a, R>)
    where
        R: Registry + 'a,
    {
    }

    fn flush<R>(&mut self, _world: SendableWorld<'a, R>)
    where
        R: Registry + 'a,
    {
    }
}

impl<'a, S, P, L> Stages<'a> for (Stage<S, P>, L)
where
    S: System<'a> + Send,
    P: ParSystem<'a> + Send,
    L: Stages<'a>,
{
    fn run<R>(&mut self, world: SendableWorld<'a, R>)
    where
        R: Registry + 'a,
    {
        self.defer(world);
        self.run_current(world);
    }

    fn defer<R>(&mut self, world: SendableWorld<'a, R>)
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

    fn run_current<R>(&mut self, world: SendableWorld<'a, R>)
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
                    }
                );
            }
            Stage::Flush => {
                self.1.flush(world);
            }
        }
    }

    fn run_continuing<R>(&mut self, world: SendableWorld<'a, R>)
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
                    }
                );
            }
            Stage::Flush => {}
        }
    }

    fn flush<R>(&mut self, world: SendableWorld<'a, R>)
    where
        R: Registry + 'a,
    {
        match &mut self.0 {
            Stage::Start(task) => {
                task.flush(world);
            }
            Stage::Continue(task) => {
                self.1.flush(world);
                task.flush(world);
            }
            Stage::Flush => {}
        }
    }
}
