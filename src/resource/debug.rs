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
        self.1.debug(debug_map);
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

#[cfg(test)]
mod tests {
    use super::Debugger;
    use crate::resources;
    use alloc::format;
    use core::any::type_name;

    #[derive(Debug)]
    struct A(u32);
    #[derive(Debug)]
    struct B(char);
    #[derive(Debug)]
    struct C(bool);

    #[test]
    fn empty() {
        let resources = resources!();
        let debugger = Debugger(&resources);

        assert_eq!(format!("{:?}", debugger), "{}");
    }

    #[test]
    fn empty_pretty_print() {
        let resources = resources!();
        let debugger = Debugger(&resources);

        assert_eq!(format!("{:#?}", debugger), "{}");
    }

    #[test]
    fn single() {
        let resources = resources!(A(42));
        let debugger = Debugger(&resources);

        assert_eq!(
            format!("{:?}", debugger),
            format!("{{\"{}\": A(42)}}", type_name::<A>())
        );
    }

    #[test]
    fn single_pretty_print() {
        let resources = resources!(A(42));
        let debugger = Debugger(&resources);

        assert_eq!(
            format!("{:#?}", debugger),
            format!(
                "{{
    \"{}\": A(
        42,
    ),
}}",
                type_name::<A>()
            )
        );
    }

    #[test]
    fn many() {
        let resources = resources!(A(42), B('a'), C(true));
        let debugger = Debugger(&resources);

        assert_eq!(
            format!("{:?}", debugger),
            format!(
                "{{\"{}\": A(42), \"{}\": B('a'), \"{}\": C(true)}}",
                type_name::<A>(),
                type_name::<B>(),
                type_name::<C>()
            )
        );
    }

    #[test]
    fn many_pretty_print() {
        let resources = resources!(A(42), B('a'), C(true));
        let debugger = Debugger(&resources);

        assert_eq!(
            format!("{:#?}", debugger),
            format!(
                "{{
    \"{}\": A(
        42,
    ),
    \"{}\": B(
        'a',
    ),
    \"{}\": C(
        true,
    ),
}}",
                type_name::<A>(),
                type_name::<B>(),
                type_name::<C>()
            )
        );
    }
}
