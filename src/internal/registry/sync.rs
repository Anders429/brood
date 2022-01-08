use crate::{component::Component, registry::{NullRegistry, Registry}};

pub trait RegistrySync: Registry {}

impl RegistrySync for NullRegistry {}

impl<C, R> RegistrySync for (C, R) where C: Component + Sync, R: RegistrySync {}
