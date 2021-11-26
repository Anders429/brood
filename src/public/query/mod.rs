use core::marker::PhantomData;

mod filter;
mod view;

pub use filter::*;
pub use view::*;

pub struct Query<'a, V, F> where V: Views<'a>, F: Filter {
    views: V,
    filter: F,
    lifetime: PhantomData<&'a ()>,
}
