use std::ptr::{self, NonNull};
use std::ops::{Deref, DerefMut};
use std::marker::PhantomData;
use std::pin::Pin;


pub trait Pointer<T>  {
    fn new(&self, ptr: NonNull<T>) -> Self;
    
    fn as_ref(&self) -> &T;

    fn as_mut(&mut self) -> &mut T;

    fn as_non_null(&mut self) -> NonNull<T>;
}

impl<T> Pointer<T> for NonNull<T> {
    fn new(&self, ptr: NonNull<T>) -> Self {
	ptr
    }
    
    fn as_ref(&self) -> &T {
	unsafe { &*self.as_ptr() }
    }

    fn as_mut(&mut self) -> &mut T {
	unsafe { &mut *self.as_ptr() }
    }

    fn as_non_null(&mut self) -> NonNull<T> {
	unsafe {
	    NonNull::new_unchecked(self.as_ptr())
	}
    }
}

#[derive(Debug)]
pub(crate) struct NonNullPtr<T, P>
where
    P: Pointer<T>
{
    ptr: P,
    _marker: PhantomData<T>,
}

impl<T, P> NonNullPtr<T, P>
where
    T: Unpin,
    P: Pointer<T>,
{
    pub fn pin(ptr: P) -> Pin<Self> {
	Pin::new(NonNullPtr {
	    ptr: ptr,
	    _marker: PhantomData,
	})
    }

    pub fn cast(ptr: &Option<Pin<Self>>) -> &P {
	let ptr = ptr as *const Option<Pin<Self>>;
	let ptr = ptr as *const P;
	unsafe { &*ptr }
    }

    pub fn as_ptr(ptr: &Option<Pin<Self>>) -> *const T {
	if let Some(ptr) = ptr {
	    let ptr: *const T = ptr.as_ref().get_ref();
	    ptr
	} else {
	    ptr::null()
	}
    }

    pub fn as_mut_ptr(ptr: &mut Option<Pin<Self>>) -> *mut T {
	if let Some(ptr) = ptr {
	    let ptr: *mut T = ptr.as_mut().get_mut();
	    ptr
	} else {
	    ptr::null_mut()
	}
    }

    pub fn as_raw(ptr: &mut Option<Pin<Self>>) -> Option<NonNull<T>> {
	if let Some(ptr) = ptr {
	    let ptr: *mut T = ptr.as_mut().get_mut();
	    Some(unsafe {NonNull::new_unchecked(ptr) })
	} else {
	    None
	}
    }
}

impl<T, P> Deref for NonNullPtr<T, P>
where
    P: Pointer<T>
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
	self.ptr.as_ref()
    }
}

impl<T, P> DerefMut for NonNullPtr<T, P>
where
    P: Pointer<T>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
	self.ptr.as_mut()
    }
}
