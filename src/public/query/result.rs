#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NullResult;

#[macro_export]
macro_rules! result {
    () => {
        _
    };
    ($component:ident $(,$components:ident)* $(,)?) => {
        ($component, result!($($components,)*))
    };
}
