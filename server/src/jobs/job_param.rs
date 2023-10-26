// use std::{
//     any::{Any, TypeId},
//     cell::{Ref, RefCell, RefMut},
//     collections::HashMap,
//     marker::PhantomData,
//     ops::{Deref, DerefMut},
// };
//
// pub(crate) trait JobParam {
//     type Item<'a>;
//     fn retrieve<'r>(resources: &'r HashMap<TypeId, RefCell<Box<dyn Any>>>) -> Self::Item<'r>;
// }
//
// pub(crate) struct Res<'a, T: 'static> {
//     value: Ref<'a, Box<dyn Any>>,
//     _marker: PhantomData<&'a T>,
// }
//
// pub(crate) struct ResMut<'a, T: 'static> {
//     value: RefMut<'a, Box<dyn Any>>,
//     _marker: PhantomData<&'a T>,
// }
//
// impl<T: 'static> Deref for Res<'_, T> {
//     type Target = T;
//
//     fn deref(&self) -> &Self::Target {
//         self.value.downcast_ref().unwrap()
//     }
// }
//
// impl<T: 'static> Deref for ResMut<'_, T> {
//     type Target = T;
//
//     fn deref(&self) -> &Self::Target {
//         self.value.downcast_ref().unwrap()
//     }
// }
//
// impl<T: 'static> DerefMut for ResMut<'_, T> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         self.value.downcast_mut().unwrap()
//     }
// }
//
// impl<'res, T: 'static> JobParam for Res<'res, T> {
//     type Item<'a> = Res<'res, T>;
//
//     fn retrieve<'r>(resources: &'r HashMap<TypeId, RefCell<Box<dyn Any>>>) -> Self::Item<'r> {
//         Res {
//             value: resources.get(&TypeId::of::<T>()).unwrap().borrow(),
//             _marker: PhantomData,
//         }
//     }
// }
//
// impl<'res, T: 'static> JobParam for ResMut<'res, T> {
//     type Item<'a> = ResMut<'res, T>;
//
//     fn retrieve<'r>(resources: &'r HashMap<TypeId, RefCell<Box<dyn Any>>>) -> Self::Item<'r> {
//         ResMut {
//             value: resources.get(&TypeId::of::<T>()).unwrap().borrow_mut(),
//             _marker: PhantomData,
//         }
//     }
// }
