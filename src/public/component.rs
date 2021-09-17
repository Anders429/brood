use core::any::Any;

pub trait Component: Any {}

impl<C> Component for C where C: Any {}
