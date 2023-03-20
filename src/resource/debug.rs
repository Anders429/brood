use crate::resource::Null;
use core::{
    any::type_name,
    fmt,
    fmt::DebugMap,
};

/// A list of resources that all implement [`Debug`].
///
/// This is a supertrait to the `Debug` trait. It is always implemented when all resources
/// implement `Debug`.
///
/// [`Debug`]: core::fmt::Debug
pub trait Debug: Sealed {}

impl Debug for Null {}

impl<Resource, Resources> Debug for (Resource, Resources)
where
    Resource: fmt::Debug,
    Resources: Debug,
{
}

pub trait Sealed {
    fn debug(&self, debug_map: &mut DebugMap);
}

impl Sealed for Null {
    fn debug(&self, _debug_map: &mut DebugMap) {}
}

impl<Resource, Resources> Sealed for (Resource, Resources)
where
    Resource: fmt::Debug,
    Resources: Sealed,
{
    fn debug(&self, debug_map: &mut DebugMap) {
        debug_map.entry(&type_name::<Resource>(), &self.0);
    }
}

pub(crate) struct Debugger<'a, Resources>(pub(crate) &'a Resources);

impl<Resources> fmt::Debug for Debugger<'_, Resources>
where
    Resources: Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug_map = formatter.debug_map();
        self.0.debug(&mut debug_map);
        debug_map.finish()
    }
}
