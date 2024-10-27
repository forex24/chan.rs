pub trait Indexable {
    fn index(&self) -> usize;
}

pub trait HasParent {
    type Parent;
    fn parent(&self) -> Option<UnsafeHandle<Self::Parent>>;
    fn set_parent(&mut self, parent: UnsafeHandle<Self::Parent>);
}

pub trait AsHandle {
    type Output;
    fn as_handle(&self) -> Self::Output;
}

// Vec扩容和收缩会导致Vec元素在内存中移动，从而直接引用元素会引发失效问题
// Handle类型是为了解决Vec元素的引用问题
// 通过Box可以保证Vec类型在堆上地址稳定的问题
// 通过index索引可以确保元素引用稳定
// 使用条件：
// 1.基于index的Vec必须是Append-only的Vec
// 2.一旦BoxedVec被drop，所有的handle都将失效，要确保Handle的生命周期小于BoxedVec生命周期
// TODO: 今后考虑用Pin
#[derive(Debug, PartialEq, Eq)]
pub struct UnsafeHandle<T> {
    ptr: *const Vec<T>, // * const 没有所有权/借用/生命周期,因此不会被Drop
    index: usize,
}

impl<T> UnsafeHandle<T> {
    // 必须是boxed vec, 不能是vec,因为vec的地址可能被移动
    pub fn new(boxed_vec: &Box<Vec<T>>, index: usize) -> Self {
        Self {
            ptr: &**boxed_vec,
            index,
        }
    }

    pub fn new_with(ptr: *const Vec<T>, index: usize) -> Self {
        Self { ptr, index }
    }

    // 获取Vec的引用
    #[inline(always)]
    pub fn parent(&self) -> &Vec<T> {
        unsafe { &*self.ptr }
    }

    #[inline(always)]
    pub fn index(&self) -> usize {
        self.index
    }

    // 向后查找相邻元素
    pub fn next_step_by(&self, step: usize) -> Option<UnsafeHandle<T>> {
        let vec: &Vec<T> = self.parent();
        if self.index + step >= vec.len() {
            None
        } else {
            Some(Self {
                ptr: self.ptr,
                index: self.index + step,
            })
        }
    }

    // 向前查找相邻元素
    pub fn prev_step_by(&self, step: usize) -> Option<UnsafeHandle<T>> {
        if step > self.index {
            None
        } else {
            Some(Self {
                ptr: self.ptr,
                index: self.index - step,
            })
        }
    }
}

// 如果T没有实现Clone,默认Handle也不会实现Clone
// 所以这里要强制实现Clone
impl<T> Clone for UnsafeHandle<T> {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            index: self.index,
        }
    }
}

// 如果T没有实现Copy,默认Handle也不会实现Copy
// 所以这里要强制实现Copy
impl<T> Copy for UnsafeHandle<T> {}

// 通过解引用使Handle变成智能指针
impl<T> std::ops::Deref for UnsafeHandle<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let vec: &Vec<T> = self.parent();
        &vec[self.index]
    }
}
