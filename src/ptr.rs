use std::fmt;
use std::fmt::Formatter;
use std::marker::{PhantomData, PhantomPinned};
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::ptr::NonNull;

pub trait Pointer<T>: fmt::Debug {
    fn from_raw(raw_ptr: NonNull<T>, self_addr: usize) -> Self;

    fn as_ref(&self) -> &T;

    fn as_mut(&mut self) -> &mut T;
}

impl<T> Pointer<T> for NonNull<T> {
    fn from_raw(raw_ptr: NonNull<T>, _: usize) -> Self {
        raw_ptr
    }

    fn as_ref(&self) -> &T {
        unsafe { &*self.as_ptr() }
    }

    fn as_mut(&mut self) -> &mut T {
        unsafe { &mut *self.as_ptr() }
    }
}

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
    pub fn assign(self_: &mut Option<Pin<Self>>, data: NonNull<T>) {
        let ptr = self_ as *mut Option<Pin<Self>>;
        let ptr = P::from_raw(data, ptr.addr());
        let ptr = NonNullPtr {
            ptr: ptr,
            _pin: PhantomPinned,
            _marker: PhantomData,
        };
        *self_ = Some(Pin::new(ptr));
    }

    pub fn assign_pin(self_: &mut Option<Pin<Self>>, data: &mut Pin<Self>) {
        Self::assign(self_, NonNull::from(data.as_mut().get_mut()))
    }

    pub fn assign_ptr(self_: &mut Option<Pin<Self>>, data: &mut Option<Pin<Self>>) {
        if let Some(data) = data {
            Self::assign_pin(self_, data)
        } else {
            *self_ = None;
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

impl<T, P> fmt::Debug for NonNullPtr<T, P>
where
    P: fmt::Debug,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self.ptr)
    }
}
