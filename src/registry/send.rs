use crate::{
    component::Component,
    registry::{Null, Registry},
};

pub trait RegistrySend: Registry {}

impl RegistrySend for Null {}

impl<C, R> RegistrySend for (C, R)
where
    C: Component + Send,
    R: RegistrySend,
{
}

#[cfg(test)]
mod tests {
    use crate::registry;

    fn is_send<R>() where R: Send {}

    #[test]
    fn empty() {
        type Registry = registry!();

        is_send::<Registry>();
    }
}
