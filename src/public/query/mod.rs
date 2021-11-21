mod filter;
mod view;

pub use filter::*;
pub use view::*;

pub struct Query<V, F> where V: Views, F: Filter {
    views: V,
    filter: F,
}
