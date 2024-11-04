use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

//use crate::{Bi::Bi::CBi, Seg::Seg::CSeg};

//use super::handle::UnsafeHandle;

//pub type Handle<T> = UnsafeHandle<T>;
pub type Handle<T> = Rc<RefCell<T>>;
pub type WeakHandle<T> = Weak<RefCell<T>>;

//pub enum LineType {
//    Bi(Handle<CBi>),
//    Seg(Handle<CSeg<CBi>>),
//}
