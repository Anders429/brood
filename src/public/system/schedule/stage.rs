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
{
}

#[macro_export]
macro_rules! stages {
    // TODO: Right now everything is comma separated. It would be ideal to have system and
    // par_system separated with another delimiter, but I don't want to deal with trying to get the
    // macro to accept multiple delimiters in the token tree at the moment.
    ($($tokens:tt),* $(,)?) => {
        stages!(internal @ $crate::system::schedule::stage::Null; $($tokens,)*)
    };
    (internal @ $processed:ty; system, $system:ty, $($remaining:tt),* $(,)?) => {
        stages!(internal @ ($crate::system::schedule::stage::Stage<$system, $crate::system::Null>, $processed); $($remaining,)*)
    };
    (internal @ $processed:ty; par_system, $par_system:ty, $($remaining:tt),* $(,)?) => {
        stages!(internal @ ($crate::system::schedule::stage::Stage<$crate::system::Null, $par_system>, $processed); $($remaining,)*)
    };
    (internal @ $processed:ty; flush, $($remaining:tt),* $(,)?) => {
        stages!(internal @ ($crate::system::schedule::stage::Stage<$crate::system::Null, $crate::system::Null>, $processed); $($remaining,)*)
    };
    (internal @ $processed:ty; $($remaining:tt),* $(,)?) => {
        $processed
    };
}
