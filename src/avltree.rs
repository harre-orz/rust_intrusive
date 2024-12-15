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
    height: usize,
    left_ptr: Option<Pin<NonNullPtr<T, P>>>,
    right_ptr: Option<Pin<NonNullPtr<T, P>>>,
}

pub struct AVLTree<T, A, P = NonNull<T>>
where
    T: Unpin,
    P: Pointer<T>,
{
    link: Link<T, P>,
    size: A,
}

impl<T, A, P> AVLTree<T, A, P>
where
    T: Unpin + Ord,
    P: Pointer<T>,
{
    pub fn insert(self: Pin<&mut Self>, data: NonNull<T>) -> bool
    {
	false
    }
}

pub struct AVLMultiTree<T, A, P = NonNull<T>>
where
    T: Unpin,
    P: Pointer<T>,
{
    link: Link<T, P>,
    size: A,
}

