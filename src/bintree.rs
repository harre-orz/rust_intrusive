use super::Adapter;
use crate::ptr::{NonNullPtr, Pointer};
use std::marker::PhantomData;
use std::pin::Pin;
use std::ptr::NonNull;

pub struct Link<T, P>
where
    T: Unpin,
    P: Pointer<T>,
{
    top_ptr: Option<Pin<NonNullPtr<T, P>>>,
    left_ptr: Option<Pin<NonNullPtr<T, P>>>,
    right_ptr: Option<Pin<NonNullPtr<T, P>>>,
}

impl<T, P> Link<T, P>
where
    T: Unpin,
    P: Pointer<T>,
{
    pub const fn new() -> Self {
        Self {
            top_ptr: None,
            left_ptr: None,
            right_ptr: None,
        }
    }

    pub const fn is_linked(&self) -> bool {
        self.left_ptr.is_some() || self.right_ptr.is_some()
    }

    pub unsafe fn unlink(&mut self) {
        self.top_ptr = None;
        self.left_ptr = None;
        self.right_ptr = None;
    }
}

impl<T, P> Default for Link<T, P>
where
    T: Unpin,
    P: Pointer<T>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, P> Unpin for Link<T, P>
where
    T: Unpin,
    P: Pointer<T>,
{
}

pub struct BinTree<T, A, P = NonNull<T>>
where
    T: Unpin,
    P: Pointer<T>,
{
    link: Link<T, P>,
    size: A,
}

impl<T, A, P> BinTree<T, A, P>
where
    T: Unpin,
    P: Pointer<T>,
    A: Adapter<T, Link = Link<T, P>>,
{
    pub fn new(_: A) -> Self {
        Self {
            link: Link::default(),
            size: A::default(),
        }
    }

    fn search(&self, data: &T) {}

    pub fn get(self: Pin<&Self>, data: &T) -> Option<Pin<&T>>
    where
        T: Ord,
    {
        None
    }

    pub fn get_mut(self: Pin<&mut Self>, data: &T) -> Option<Pin<&mut T>>
    where
        T: Ord,
    {
        None
    }

    pub fn insert(self: Pin<&mut Self>, data: NonNull<T>) -> Option<NonNull<T>>
    where
        T: Ord,
    {
        None
    }
}

impl<T, A, P> Default for BinTree<T, A, P>
where
    T: Unpin,
    P: Pointer<T>,
    A: Adapter<T, Link = Link<T, P>>,
{
    fn default() -> Self {
        Self::new(A::default())
    }
}

impl<T, A, P> Unpin for BinTree<T, A, P>
where
    T: Unpin,
    P: Pointer<T>,
{
}
