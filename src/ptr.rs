use std::fmt::Debug;
use std::marker::{PhantomData, PhantomPinned};
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::ptr::NonNull;

pub trait Pointer<T> {
    fn from_raw(raw_ptr: NonNull<T>, base: usize) -> Self;

    fn as_ref(&self) -> &T;

    fn as_mut(&mut self) -> &mut T;
}

impl<T> Pointer<T> for NonNull<T> {
    fn from_raw(raw_ptr: NonNull<T>, _: usize) -> Self
    {
        raw_ptr
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
    pub fn assign(this: &mut Option<Pin<Self>>, data: NonNull<T>)
    {
        let ptr = this as *mut Option<Pin<Self>>;
        let ptr = P::from_raw(data, ptr.addr());
        let ptr = NonNullPtr {
            ptr: ptr,
            _pin: PhantomPinned,
            _marker: PhantomData,
        };
        *this = Some(Pin::new(ptr));
    }

    pub fn assign_pin(this: &mut Option<Pin<Self>>, data: &mut Pin<Self>)
    {
        Self::assign(this, NonNull::from(data.as_mut().get_mut()))
    }

    pub fn assign_ptr(this: &mut Option<Pin<Self>>, data: &mut Option<Pin<Self>>)
    {
        if let Some(data) = data {
            Self::assign_pin(this, data)
        } else {
            *this = None;
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
