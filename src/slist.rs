use super::{Sized, Adapter};
use crate::ptr::{NonNullPtr, Pointer};
use std::pin::Pin;
use std::ptr::NonNull;
use std::marker::{PhantomData, PhantomPinned};

#[derive(Debug)]
pub struct Link<T, P = NonNull<T>>
where
    P: Pointer<T>,
{
    next_ptr: Option<Pin<NonNullPtr<T, P>>>,
    _pin: PhantomPinned,
    _marker: PhantomData<T>,
}

impl<T, P> Unpin for Link<T, P>
where
    P: Pointer<T>,
{}

impl<T, P> Link<T, P>
where
    P: Pointer<T>,
{
    pub fn new() -> Self {
	Self {
	    next_ptr: None,
	    _pin: PhantomPinned,
	    _marker: PhantomData,
	}
    }

    pub fn is_linked(&self) -> bool {
	self.next_ptr.is_some()
    }

    pub fn unlink(&mut self) {
	self.next_ptr = None;
    }
}

pub struct Iter<'a, T, P, A>
where
    P: Pointer<T>,
{
    item_ptr: *const T,
    _marker: PhantomData<(&'a (), P, A)>,
}

impl<'a, T: 'a, P, A> Iterator for Iter<'a, T, P, A>
where
    T: Unpin,
    P: Pointer<T>,
    A: Adapter<T, Link = Link<T, P>>,
{
    type Item = Pin<&'a T>;

    fn next(&mut self) -> Option<Self::Item> {
	if self.item_ptr.is_null() {
	    None
	} else {
	    let item = unsafe { &*self.item_ptr };
	    let item_link = A::as_link_ref(item);
	    self.item_ptr = NonNullPtr::as_ptr(&item_link.next_ptr);
	    Some(Pin::new(item))
	}
    }
}

#[derive(Debug)]
pub struct SinglyLinkedList<T, A, P = NonNull<T>>
where
    A: Adapter<T, Link = Link<T, P>>,
    P: Pointer<T>,
{
    head: Link<T, P>,
    size: A,
}

impl<T, A, P> Unpin for SinglyLinkedList<T, A, P>
where
    A: Adapter<T, Link = Link<T, P>>,
    P: Pointer<T>,
{}

impl<T, A, P> SinglyLinkedList<T, A, P>
where
    T: Unpin,
    A: Adapter<T, Link = Link<T, P>>,
    P: Pointer<T>,
{
    pub fn new(_: A) -> Self {
	Self {
	    head: Link::new(),
	    size: A::default(),
	}
    }
    
    pub fn push_front(self: Pin<&mut Self>, mut data: NonNull<T>) {
	let data_link: &mut Link<T, P> = A::as_link_mut(unsafe { data.as_mut() });
	debug_assert_eq!(data_link.is_linked(), false);
	
	let head_ptr = &mut Pin::into_inner(self).head.next_ptr;
	if let Some(first) = head_ptr {
	    let first = unsafe { NonNull::new_unchecked(first.as_mut().get_mut()) };
	    let first = NonNullPtr::pin(NonNullPtr::cast(&data_link.next_ptr).new(first));
	    data_link.next_ptr = Some(first);
	}
	let data = NonNullPtr::pin(NonNullPtr::cast(head_ptr).new(data));
	*head_ptr = Some(data);
    }

    pub fn pop_front(self: Pin<&mut Self>) -> Option<NonNull<T>> {
	let head_ptr = &mut Pin::into_inner(self).head.next_ptr;
	if let Some(first) = head_ptr {
	    let mut data = unsafe { NonNull::new_unchecked(first.as_mut().get_mut()) };
	    let first_link: &mut Link<T, P> = A::as_link_mut(first);
	    if let Some(first_next) = NonNullPtr::as_raw(&mut first_link.next_ptr) {
		let first = NonNullPtr::pin(NonNullPtr::cast(&head_ptr).new(first_next));
	     	*head_ptr = Some(first);
	    } else {
	     	*head_ptr = None;
	    }
	    A::as_link_mut(unsafe { data.as_mut() } ).unlink();
	    Some(data)
	} else {
	    None
	}
    }

    pub fn front(self: Pin<&Self>) -> Option<Pin<&T>> {
	let head_ptr = &Pin::into_inner(self).head.next_ptr;
	if let Some(first) = head_ptr {
	    let first: &Pin<NonNullPtr<T, P>> = first;
	    let first: Pin<&T> = first.as_ref();
	    Some(first)
	} else {
	    None
	}
    }

    pub fn iter(self: Pin<&Self>) -> Iter<T, P, A> {
	let head_ptr = &self.head.next_ptr;
	Iter {
	    item_ptr: NonNullPtr::as_ptr(head_ptr),
	    _marker: PhantomData,
	}
    }
}



#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug)]
    struct X {
	x: i32,
	link: Link<Self>,
    }

    impl X {
	pub fn new(x: i32) -> NonNull<Self> {
	    let ptr = Box::new(X {
		x: x,
		link: Link::new(),
	    });
	    let ptr = Box::into_raw(ptr);
	    NonNull::new(ptr).unwrap()
	}
    }

    #[derive(Default, Debug)]
    struct XLink;

    impl Sized for XLink {}

    impl Adapter<X> for XLink {
	type Link = Link<X>;

	fn as_link_ref(data: &X) -> &Self::Link {
	    &data.link
	}

	fn as_link_mut(data: &mut X) -> &mut Self::Link {
	    &mut data.link
	}
    }

    #[test]
    fn test() {

	let mut lst = Box::pin(SinglyLinkedList::new(XLink));
	lst.as_mut().push_front(X::new(1));
	lst.as_mut().push_front(X::new(2));

	let ptr = lst.as_mut().pop_front().unwrap();
	assert_eq!(unsafe { Box::from_raw(ptr.as_ptr()) }.x, 2);
	
	let ptr = lst.as_mut().pop_front().unwrap();
	assert_eq!(unsafe { Box::from_raw(ptr.as_ptr()) }.x, 1);

	assert_eq!(lst.as_mut().pop_front(), None);
    }
}
