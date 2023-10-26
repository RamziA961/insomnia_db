// use super::job_param::{JobParam, ResMut};
// use std::{
//     any::{Any, TypeId},
//     cell::RefCell,
//     collections::HashMap,
//     marker::PhantomData,
// };
//
// pub(crate) trait Job {
//     fn run(&mut self, resources: &mut HashMap<TypeId, RefCell<Box<dyn Any>>>);
// }
//
// pub(crate) trait IntoJob {
//     type Job: Job;
//     fn into_job(self) -> Self::Job;
// }
//
// pub(crate) struct JobTask<Input, F> {
//     //    need to use dependency injection
//     //    should allow support variadic functions
//     //    do we want to return a Result? should it be a future or blocking?
//     //    https://promethia-27.github.io/dependency_injection_like_bevy_from_scratch/introductions.html
//     task: F,
//     _marker: PhantomData<fn() -> Input>,
// }
//
// macro_rules! impl_job {
//     ($($params:ident),*) => {
//         #[allow(non_snake_case)]
//         #[allow(unused)]
//         impl<F, $($params: JobParam),*> Job for JobTask<($($params,)*), F>
//         where
//             for<'a, 'b> &'a mut F: FnMut($($params),*) + FnMut($(<$params as JobParam>::Item<'b>),*)
//         {
//             fn run(&mut self, resources: &mut HashMap<TypeId, RefCell<Box<dyn Any>>>) {
//                 fn call_inner<$($params),*>(mut f: impl FnMut($($params),*), $($params: $params),*) {
//                     f($($params),*)
//                 }
//
//                 $(let $params = $params::retrieve(resources);)*
//                 call_inner(&mut self.task, $($params),*)
//             }
//         }
//     }
// }
//
// macro_rules! impl_into_job {
//     ($($params:ident),*) => {
//         //impl<F, $($params: JobParam),*> IntoJob<$($params,)*> for F
//         impl<F, $($params: JobParam),*> IntoJob for F
//         where
//             for<'a, 'b> &'a mut F: FnMut($($params),*) + FnMut($(<$params as JobParam>::Item<'b>),*)
//         {
//             type Job = JobTask<($($params,)*), Self>;
//
//             fn into_job(self) -> Self::Job {
//                 JobTask {
//                     task: self,
//                     _marker: Default::default()
//                 }
//             }
//         }
//     }
// }
//
// impl_job!();
// impl_job!(T1);
// impl_job!(T1, T2);
// impl_job!(T1, T2, T3);
// impl_job!(T1, T2, T3, T4);
//
// impl_into_job!();

use crate::database::shared_state::SharedState;
use std::{marker::PhantomData, sync::Arc};

pub(crate) struct JobTask<F, I> {
    task: F,
    _marker: PhantomData<fn() -> I>,
}

pub(crate) trait Job {
    fn run(&mut self, resources: Arc<SharedState>) -> Result<(), ()>;
}

pub(crate) trait IntoJob<F, I>
where
    for<'f> &'f mut F: FnMut(Arc<SharedState>) -> Result<(), ()>,
{
    fn into_job(self) -> JobTask<F, I>;
}

impl<F, I> Job for JobTask<F, I>
where
    for<'f> &'f mut F: FnMut(Arc<SharedState>) -> Result<(), ()>,
{
    fn run(&mut self, resources: Arc<SharedState>) -> Result<(), ()> {
        fn call_inner(
            mut f: impl FnMut(Arc<SharedState>) -> Result<(), ()>,
            resources: Arc<SharedState>,
        ) -> Result<(), ()> {
            f(resources)
        }

        call_inner(&mut self.task, resources)
    }
}

impl<F, I> IntoJob<F, I> for F
where
    for<'f> &'f mut F: FnMut(Arc<SharedState>) -> Result<(), ()>,
{
    fn into_job(self) -> JobTask<Self, I> {
        JobTask {
            task: self,
            _marker: Default::default(),
        }
    }
}
