use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

//use crate::{Bi::Bi::CBi, Seg::Seg::CSeg};

//use super::handle::UnsafeHandle;

//pub type Handle<T> = UnsafeHandle<T>;
pub type StrongHandle<T> = Rc<RefCell<T>>;
pub type WeakHandle<T> = Weak<RefCell<T>>;

//pub type Handle<T> = StrongHandle<T>;

//pub enum LineType {
//    Bi(Handle<CBi>),
//    Seg(Handle<CSeg<CBi>>),
//}
