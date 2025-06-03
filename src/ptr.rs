use std::fmt::Debug;
use std::marker::{PhantomData, PhantomPinned};
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::ptr::NonNull;

pub trait Pointer<T> {
    fn assign(self: &mut Self, raw_ptr: NonNull<T>);

    fn as_ref(&self) -> &T;

    fn as_mut(&mut self) -> &mut T;
}

impl<T> Pointer<T> for NonNull<T> {
    fn assign(self: &mut Self, raw_ptr: NonNull<T>) {
        *self = raw_ptr;
    }

    fn as_ref(&self) -> &T {
        unsafe { &*self.as_ptr() }
    }

    fn as_mut(&mut self) -> &mut T {
        unsafe { &mut *self.as_ptr() }
    }
}

#[derive(Debug)]
pub(crate) struct NonNullPtr<T, P> {
    ptr: P,
    _pin: PhantomPinned,
    _marker: PhantomData<T>,
}

impl<T, P> NonNullPtr<T, P>
where
    T: Unpin,
    P: Pointer<T>,
{
    pub fn assign(self_: &mut Option<Pin<Self>>, raw_ptr: NonNull<T>) {
        let self_ptr = self_ as *mut Option<Pin<Self>> as *mut P;
        P::assign(unsafe { &mut *self_ptr }, raw_ptr);
    }

    pub fn as_raw_ptr(ptr: &mut Option<Pin<Self>>) -> Option<NonNull<T>> {
        if let Some(ptr) = ptr {
            let ptr = ptr.as_mut().get_mut();
            Some(unsafe { NonNull::new_unchecked(ptr) })
        } else {
            None
        }
    }
}

impl<T, P> Deref for NonNullPtr<T, P>
where
    P: Pointer<T>,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.ptr.as_ref()
    }
}

impl<T, P> DerefMut for NonNullPtr<T, P>
where
    P: Pointer<T>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ptr.as_mut()
    }
}
