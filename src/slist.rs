use super::Adapter;
use crate::ptr::{NonNullPtr, Pointer};
use std::marker::PhantomData;
use std::pin::Pin;
use std::ptr::NonNull;

#[derive(Debug)]
pub struct Link<T, P = NonNull<T>>
where
    T: Unpin,
    P: Pointer<T>,
{
    next_ptr: Option<Pin<NonNullPtr<T, P>>>,
}

impl<T, P> Link<T, P>
where
    T: Unpin,
    P: Pointer<T>,
{
    pub const fn new() -> Self {
        Self { next_ptr: None }
    }

    pub const fn is_linked(&self) -> bool {
        self.next_ptr.is_some()
    }

    unsafe fn unlink(&mut self) {
        self.next_ptr = None;
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

pub struct Iter<'a, T, A, P>
where
    T: Unpin,
    P: Pointer<T>,
{
    link_ptr: *const Link<T, P>,
    _marker: PhantomData<(&'a (), A, P)>,
}

impl<'a, T: 'a, A, P> Iterator for Iter<'a, T, A, P>
where
    T: Unpin,
    P: Pointer<T>,
    A: Adapter<T, Link = Link<T, P>>,
{
    type Item = Pin<&'a T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.link_ptr.is_null() {
            None
        } else {
	    let next_ptr = &unsafe { &*self.link_ptr }.next_ptr;
	    let item = unsafe { &*NonNullPtr::as_ptr(next_ptr) };
	    self.link_ptr = A::as_link_ref(item);
	    Some(Pin::new(item))
        }
    }
}

pub struct IterMut<'a, T, A, P>
where
    T: Unpin,
    P: Pointer<T>,
{
    link_ptr: *mut Link<T, P>,
    _marker: PhantomData<&'a A>,
}

impl<'a, T: 'a, A, P> Iterator for IterMut<'a, T, A, P>
where
    T: Unpin,
    P: Pointer<T>,
    A: Adapter<T, Link = Link<T, P>>,
{
    type Item = Pin<&'a mut T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.link_ptr.is_null() {
            None
        } else {
	    let next_ptr = &mut unsafe { &mut *self.link_ptr }.next_ptr;
	    let item = unsafe { &mut *NonNullPtr::as_mut_ptr(next_ptr) };
	    self.link_ptr = A::as_link_mut(item);
	    Some(Pin::new(item))
        }
    }
}

pub struct IntoIter<'a, T, A, P>
where
    T: Unpin,
    P: Pointer<T>,
{
    item: Pin<&'a mut SinglyLinkedList<T, A, P>>,
}

impl<'a, T, A, P> Iterator for IntoIter<'a, T, A, P>
where
    T: Unpin,
    P: Pointer<T>,
    A: Adapter<T, Link = Link<T, P>>,
{
    type Item = NonNull<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.item.as_mut().pop_front()
    }
}


#[derive(Debug)]
pub struct SinglyLinkedList<T, A, P = NonNull<T>>
where
    T: Unpin,
    P: Pointer<T>,
{
    link: Link<T, P>,
    size: A,
}

impl<T, A, P> SinglyLinkedList<T, A, P>
where
    T: Unpin,
    P: Pointer<T>,
    A: Adapter<T, Link = Link<T, P>>,
{
    pub fn new(_: A) -> Self {
        Self::default()
    }

    pub fn push_front(self: Pin<&mut Self>, mut data: NonNull<T>) {
        let data_link = A::as_link_mut(unsafe { data.as_mut() });
        debug_assert_eq!(data_link.is_linked(), false);

        let self_ = Pin::into_inner(self);
        self_.size.increment();
        let head_ptr = &mut self_.link.next_ptr;
        if let Some(first) = head_ptr {
            let first = unsafe { NonNull::new_unchecked(first.as_mut().get_mut()) };
            NonNullPtr::set(&mut data_link.next_ptr, first);
        }
        NonNullPtr::set(head_ptr, data);
    }

    pub fn pop_front(self: Pin<&mut Self>) -> Option<NonNull<T>> {
        let self_ = Pin::into_inner(self);
        let head_ptr = &mut self_.link.next_ptr;
        if let Some(first) = head_ptr {
            self_.size.decrement();
            let mut data = unsafe { NonNull::new_unchecked(first.as_mut().get_mut()) };
            let first_link = A::as_link_mut(first);
            if let Some(first_next) = NonNullPtr::as_raw_ptr(&mut first_link.next_ptr) {
                NonNullPtr::set(head_ptr, first_next);
            } else {
                *head_ptr = None;
            }
            unsafe {
                A::as_link_mut(data.as_mut()).unlink();
            }
            Some(data)
        } else {
            None
        }
    }

    pub fn front(self: Pin<&Self>) -> Option<Pin<&T>> {
        let self_ = Pin::into_inner(self);
        if let Some(first) = &self_.link.next_ptr {
            Some(first.as_ref())
        } else {
            None
        }
    }

    pub fn front_mut(self: Pin<&mut Self>) -> Option<Pin<&mut T>> {
        let self_ = Pin::into_inner(self);
        if let Some(first) = &mut self_.link.next_ptr {
            Some(first.as_mut())
        } else {
            None
        }
    }

    pub fn iter(self: Pin<&Self>) -> Iter<T, A, P> {
        let self_ = Pin::into_inner(self);
        Iter {
            link_ptr: &self_.link,
            _marker: PhantomData,
        }
    }

    pub fn iter_mut(self: Pin<&mut Self>) -> IterMut<T, A, P> {
        let self_ = Pin::into_inner(self);
        IterMut {
            link_ptr: &mut self_.link,
            _marker: PhantomData,
        }
    }

    pub const fn into_iter(self: Pin<&mut Self>) -> IntoIter<T, A, P> {
        IntoIter { item: self }
    }

    pub fn is_empty(self: Pin<&Self>) -> bool {
        self.size.is_empty(self.iter())
    }

    pub fn count(self: Pin<&Self>) -> usize {
        self.size.count(self.iter())
    }

    pub fn contains(self: Pin<&Self>, data: &T) -> bool
    where
	T: PartialEq<T>,
    {
	for it in self.iter() {
	    if data.eq(it.get_ref()) {
		return true
	    }
	}
	false
    }

    pub fn append(mut self: Pin<&mut Self>, other: Pin<&mut Self>) {
	let vec: Vec<_> = other.into_iter().collect();
	for data in vec.into_iter().rev() {
	    self.as_mut().push_front(data);
	}
    }
}

impl<T, A, P> Default for SinglyLinkedList<T, A, P>
where
    T: Unpin,
    P: Pointer<T>,
    A: Default,
{
    fn default() -> Self {
        Self {
            link: Link::new(),
            size: A::default(),
        }
    }
}

impl<T, A, P> Unpin for SinglyLinkedList<T, A, P>
where
    T: Unpin,
    P: Pointer<T>,
{
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Size;

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

    impl Size for XLink {}

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
        let ptr = unsafe { Box::from_raw(ptr.as_ptr()) };
        assert_eq!(ptr.link.is_linked(), false);
        assert_eq!(ptr.x, 2);

        let ptr = lst.as_mut().pop_front().unwrap();
        let ptr = unsafe { Box::from_raw(ptr.as_ptr()) };
        assert_eq!(ptr.link.is_linked(), false);
        assert_eq!(ptr.x, 1);

        assert_eq!(lst.as_mut().pop_front(), None);
    }
}
