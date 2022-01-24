mod iter;
#[cfg(feature = "parallel")]
mod par_iter;

pub use iter::Iter;
#[cfg(feature = "parallel")]
pub use par_iter::ParIter;

use crate::hlist::define_null;

define_null!();

#[macro_export]
macro_rules! result {
    () => {
        _
    };
    ($component:ident $(,$components:ident)* $(,)?) => {
        ($component, result!($($components,)*))
    };
}
