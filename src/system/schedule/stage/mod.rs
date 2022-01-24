mod seal;

use crate::system::{schedule::task::Task, ParSystem, System};
use seal::Seal;

pub enum Stage<S, P> {
    Start(Task<S, P>),
    Continue(Task<S, P>),
    Flush,
}

pub trait Stages<'a>: Seal<'a> {}

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
    ($($idents:tt $(: $systems:tt)?),* $(,)?) => {
        stages!(internal @ $crate::system::schedule::stage::Null; $($idents $(: $systems)?,)*)
    };
    (internal @ $processed:ty; system: $system:ty, $($idents:tt $(: $systems:tt)?),* $(,)?) => {
        stages!(internal @ ($crate::system::schedule::stage::Stage<$system, $crate::system::Null>, $processed); $($idents $(: $systems)?,)*)
    };
    (internal @ $processed:ty; par_system: $par_system:ty, $($idents:tt $(: $systems:tt)?),* $(,)?) => {
        stages!(internal @ ($crate::system::schedule::stage::Stage<$crate::system::Null, $par_system>, $processed); $($idents $(: $systems)?,)*)
    };
    (internal @ $processed:ty; flush, $($idents:tt $(: $systems:tt)?),* $(,)?) => {
        stages!(internal @ ($crate::system::schedule::stage::Stage<$crate::system::Null, $crate::system::Null>, $processed); $($idents $(: $systems)?,)*)
    };
    (internal @ $processed:ty; $($idents:tt $(: $systems:tt)?),* $(,)?) => {
        $processed
    };
}
