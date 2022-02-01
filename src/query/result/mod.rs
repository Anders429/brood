mod iter;
#[cfg(feature = "parallel")]
mod par_iter;

pub use iter::Iter;
#[cfg(feature = "parallel")]
pub use par_iter::ParIter;

use crate::{doc, hlist::define_null};

define_null!();

doc::non_root_macro! {
    /// Defines identifiers to match items returned by a [`result::Iter`] iterator.
    ///
    /// This allows matching identifiers with the heterogeneous lists iterated by the `result::Iter`
    /// iterator.
    ///
    /// # Example
    /// ``` rust
    /// use brood::{entity, query::{filter, result, views}, registry, World};
    ///
    /// struct Foo(u32);
    /// struct Bar(bool);
    ///
    /// type Registry = registry!(Foo, Bar);
    ///
    /// let mut world = World::<Registry>::new();
    /// world.push(entity!(Foo(42), Bar(true)));
    ///
    /// for result!(foo, bar) in world.query::<views!(&mut Foo, &Bar), filter::None>() {
    ///     // ...
    /// }
    /// ```
    ///
    /// [`result::Iter`]: crate::query::result::Iter
    macro_rules! result {
        () => (
            _
        );
        ($component:ident $(,$components:ident)* $(,)?) => (
            ($component, $crate::query::result!($($components,)*))
        );
    }
}
