pub mod task;

mod claim;
mod scheduler;
mod sealed;
mod sendable;
mod stage;
mod stager;
mod stages;

pub(crate) use stages::Stages;

use crate::{
    doc,
    registry::Registry,
};
use scheduler::Scheduler;
use sealed::Sealed;
use stage::Stage;
use stager::Stager;
use task::Task;

pub trait Schedule<'a, R, I, P, RI, SFI, SVI, SP, SI, SQ>:
    Sealed<'a, R, I, P, RI, SFI, SVI, SP, SI, SQ>
where
    R: Registry,
{
}

impl<'a, R, T, I, P, RI, SFI, SVI, SP, SI, SQ> Schedule<'a, R, I, P, RI, SFI, SVI, SP, SI, SQ> for T
where
    R: Registry,
    T: Sealed<'a, R, I, P, RI, SFI, SVI, SP, SI, SQ>,
{
}

doc::non_root_macro! {
    macro_rules! schedule {
        ($task:expr $(,$tasks:expr)* $(,)?) => (
            ($task, $crate::system::schedule::schedule!($($tasks,)*))
        );
        () => (
            $crate::system::schedule::task::Null
        );
    }
}

#[cfg(test)]
mod tests {
    use super::Sealed as Schedule;
    use crate::{
        query::{
            filter,
            result,
            views,
        },
        registry,
        registry::ContainsQuery,
        system,
        system::{
            schedule::task,
            System,
        },
    };

    #[test]
    fn foo() {
        extern crate std;
        use std::any::type_name;

        #[derive(Clone)]
        struct A(f32);
        #[derive(Clone)]
        struct B(f32);
        #[derive(Clone)]
        struct C(f32);
        #[derive(Clone)]
        struct D(f32);
        #[derive(Clone)]
        struct E(f32);

        type Registry = registry!(A, B, C, D, E);

        struct AB;

        impl System for AB {
            type Views<'a> = views!(&'a mut A, &'a mut B);
            type Filter = filter::None;

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            ) where
                R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                for result!(a, b) in query_results {
                    core::mem::swap(&mut a.0, &mut b.0);
                }
            }
        }

        struct CD;

        impl System for CD {
            type Views<'a> = views!(&'a mut C, &'a mut D);
            type Filter = filter::None;

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            ) where
                R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                for result!(c, d) in query_results {
                    core::mem::swap(&mut c.0, &mut d.0);
                }
            }
        }

        struct CE;

        impl System for CE {
            type Views<'a> = views!(&'a mut C, &'a mut E);
            type Filter = filter::None;

            fn run<'a, R, FI, VI, P, I, Q>(
                &mut self,
                query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            ) where
                R: ContainsQuery<'a, Self::Filter, FI, Self::Views<'a>, VI, P, I, Q>,
            {
                for result!(c, e) in query_results {
                    core::mem::swap(&mut c.0, &mut e.0);
                }
            }
        }

        std::dbg!(type_name::<
            <(
                task::System<AB>,
                (task::System<CD>, (task::System<CE>, task::Null))
            ) as Schedule<'_, Registry, _, _, _, _, _, _, _, _>>::Stages,
        >());
        // MyTasks::stages(Null);

        assert!(false);
    }
}
