use crate::{
    registry::{
        ContainsParQuery,
        ContainsQuery,
        Registry,
    },
    system::{
        schedule::{
            sendable::SendableWorld,
            stage::{
                Null,
                Stage,
            },
        },
        ParSystem,
        System,
    },
    world::World,
};

pub trait Sealed<R, SFI, SVI, PFI, PVI, SP, SI, SQ, PP, PI, PQ>: Send
where
    R: Registry,
{
    fn run(&mut self, world: SendableWorld<R>);

    fn defer(&mut self, world: SendableWorld<R>);

    fn run_current(&mut self, world: SendableWorld<R>);

    fn run_continuing(&mut self, world: SendableWorld<R>);

    fn flush(&mut self, world: SendableWorld<R>);
}

impl<'a, R> Sealed<R, Null, Null, Null, Null, Null, Null, Null, Null, Null, Null> for Null
where
    R: Registry + 'a,
{
    fn run(&mut self, _world: SendableWorld<R>) {}

    fn defer(&mut self, _world: SendableWorld<R>) {}

    fn run_current(&mut self, _world: SendableWorld<R>) {}

    fn run_continuing(&mut self, _world: SendableWorld<R>) {}

    fn flush(&mut self, _world: SendableWorld<R>) {}
}

impl<
        'a,
        S,
        P,
        L,
        R,
        SFI,
        SFIS,
        SVI,
        SVIS,
        PFI,
        PFIS,
        PVI,
        PVIS,
        SP,
        SPS,
        SI,
        SIS,
        SQ,
        SQS,
        PP,
        PPS,
        PI,
        PIS,
        PQ,
        PQS,
    >
    Sealed<
        R,
        (SFI, SFIS),
        (SVI, SVIS),
        (PFI, PFIS),
        (PVI, PVIS),
        (SP, SPS),
        (SI, SIS),
        (SQ, SQS),
        (PP, PPS),
        (PI, PIS),
        (PQ, PQS),
    > for (Stage<S, P>, L)
where
    R: ContainsQuery<'a, S::Filter, SFI, S::Views<'a>, SVI, SP, SI, SQ>
        + ContainsParQuery<'a, P::Filter, PFI, P::Views<'a>, PVI, PP, PI, PQ>
        + 'a,
    S: System + Send,
    P: ParSystem + Send,
    L: Sealed<R, SFIS, SVIS, PFIS, PVIS, SPS, SIS, SQS, PPS, PIS, PQS>,
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
