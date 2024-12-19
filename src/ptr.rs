use std::marker::{PhantomData, PhantomPinned};
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::ptr::NonNull;
use std::fmt::Debug;

pub trait Pointer<T>
where
    T: Unpin,
{
    fn set(&self, ptr: NonNull<T>) -> Self;

    fn as_ref(&self) -> &T;

    fn as_mut(&mut self) -> &mut T;
}

impl<T> Pointer<T> for NonNull<T>
where
    T: Unpin,
{
    fn set(&self, ptr: NonNull<T>) -> Self {
        ptr
    }

    fn as_ref(&self) -> &T {
        unsafe { &*self.as_ptr() }
    }

    fn as_mut(&mut self) -> &mut T {
        unsafe { &mut *self.as_ptr() }
    }
}

#[derive(Debug)]
pub(crate) struct NonNullPtr<T, P>
where
    T: Unpin,
    P: Pointer<T>,
{
    ptr: P,
    _pin: PhantomPinned,
    _marker: PhantomData<T>,
}

impl<T, P> NonNullPtr<T, P>
where
    T: Unpin,
    P: Pointer<T>,
{
    pub fn set(self_: &mut Option<Pin<Self>>, ptr: NonNull<T>) {
        let self_ptr = self_ as *const Option<Pin<Self>> as *const P;
        *self_ = Some(Pin::new(NonNullPtr {
            ptr: unsafe { &*self_ptr }.set(ptr),
            _pin: PhantomPinned,
            _marker: PhantomData,
        }))
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
    T: Unpin,
    P: Pointer<T>,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.ptr.as_ref()
    }
}

impl<T, P> DerefMut for NonNullPtr<T, P>
where
    T: Unpin,
    P: Pointer<T>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ptr.as_mut()
    }
}
