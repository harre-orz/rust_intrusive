use super::LinkAdapter;
use crate::ptr::{NonNullPtr, Pointer};
use std::marker::PhantomData;
use std::pin::Pin;
use std::ptr::NonNull;

#[derive(Debug)]
pub struct Link<T, P = NonNull<T>> {
    next_ptr: Option<Pin<NonNullPtr<T, P>>>,
    prev_ptr: Option<Pin<NonNullPtr<T, P>>>,
}

impl<T, P> Link<T, P> {
    pub const fn new() -> Self {
        Self {
            next_ptr: None,
            prev_ptr: None,
        }
    }

    pub const fn is_linked(&self) -> bool {
        self.next_ptr.is_some() || self.prev_ptr.is_some()
    }

    fn unlink(&mut self) {
        self.next_ptr = None;
        self.prev_ptr = None;
    }
}

impl<T, P> Default for Link<T, P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, P> Unpin for Link<T, P> where T: Unpin {}

pub struct Iter<'a, T, A, P> {
    link: *const Link<T, P>,
    _marker: PhantomData<&'a A>,
}

impl<'a, T, A, P> Iterator for Iter<'a, T, A, P>
where
    T: Unpin + 'a,
    P: Pointer<T> + 'a,
    A: LinkAdapter<T, Link = Link<T, P>>,
{
    type Item = Pin<&'a T>;

    fn next(&mut self) -> Option<Self::Item> {
        let link = unsafe { &*self.link };
        if let Some(item) = &link.next_ptr {
            self.link = A::as_link_ref(item);
            Some(item.as_ref())
        } else {
            None
        }
    }
}

impl<'a, T, A, P> DoubleEndedIterator for Iter<'a, T, A, P>
where
    T: Unpin + 'a,
    P: Pointer<T> + 'a,
    A: LinkAdapter<T, Link = Link<T, P>>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let link = unsafe { &*self.link };
        if let Some(item) = &link.prev_ptr {
            self.link = A::as_link_ref(item);
            Some(item.as_ref())
        } else {
            None
        }
    }
}

pub struct IterMut<'a, T, A, P> {
    link: *mut Link<T, P>,
    _marker: PhantomData<&'a A>,
}

impl<'a, T, A, P> Iterator for IterMut<'a, T, A, P>
where
    T: Unpin + 'a,
    P: Pointer<T> + 'a,
    A: LinkAdapter<T, Link = Link<T, P>>,
{
    type Item = Pin<&'a mut T>;

    fn next(&mut self) -> Option<Self::Item> {
        let link = unsafe { &mut *self.link };
        if let Some(item) = &mut link.next_ptr {
            self.link = A::as_link_mut(item);
            Some(item.as_mut())
        } else {
            None
        }
    }
}

impl<'a, T, A, P> DoubleEndedIterator for IterMut<'a, T, A, P>
where
    T: Unpin + 'a,
    P: Pointer<T> + 'a,
    A: LinkAdapter<T, Link = Link<T, P>>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let link = unsafe { &mut *self.link };
        if let Some(item) = &mut link.prev_ptr {
            self.link = A::as_link_mut(item);
            Some(item.as_mut())
        } else {
            None
        }
    }
}

pub struct IntoIter<'a, T, A, P> {
    item: Pin<&'a mut DoublyLinkedList<T, A, P>>,
}

impl<'a, T, A, P> Iterator for IntoIter<'a, T, A, P>
where
    T: Unpin,
    P: Pointer<T>,
    A: LinkAdapter<T, Link = Link<T, P>>,
{
    type Item = NonNull<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.item.as_mut().pop_front()
    }
}

impl<'a, T, A, P> DoubleEndedIterator for IntoIter<'a, T, A, P>
where
    T: Unpin,
    P: Pointer<T>,
    A: LinkAdapter<T, Link = Link<T, P>>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.item.as_mut().pop_back()
    }
}

pub struct DoublyLinkedList<T, A, P> {
    link: Link<T, P>,
    size: A,
}

impl<T, A, P> DoublyLinkedList<T, A, P> {
    pub const fn new(adapter: A) -> Self {
        Self {
            link: Link::new(),
            size: adapter,
        }
    }
}

impl<T, A, P> DoublyLinkedList<T, A, P> where T: Unpin {}

impl<T, A, P> DoublyLinkedList<T, A, P>
where
    T: Unpin,
    P: Pointer<T>,
    A: LinkAdapter<T, Link = Link<T, P>>,
{
    pub fn push_front(self: Pin<&mut Self>, mut item: NonNull<T>) {
        let item_link = A::as_link_mut(unsafe { item.as_mut() });
        debug_assert_eq!(item_link.is_linked(), false);

        let self_ = Pin::into_inner(self);
        let head_ptr = &mut self_.link.next_ptr;
        let tail_ptr = &mut self_.link.prev_ptr;
        if let Some(head) = head_ptr {
            let head_link = A::as_link_mut(head.as_mut().get_mut());
            NonNullPtr::assign(&mut head_link.prev_ptr, item);
            NonNullPtr::assign_pin(&mut item_link.next_ptr, head);
        } else {
            NonNullPtr::assign(tail_ptr, item);
        }
        NonNullPtr::assign(head_ptr, item);
        self_.size.incr();
    }

    pub fn push_back(self: Pin<&mut Self>, mut item: NonNull<T>) {
        let item_link = A::as_link_mut(unsafe { item.as_mut() });
        debug_assert_eq!(item_link.is_linked(), false);

        let self_ = Pin::into_inner(self);
        let head_ptr = &mut self_.link.next_ptr;
        let tail_ptr = &mut self_.link.prev_ptr;
        if let Some(tail) = tail_ptr {
            let tail_link = A::as_link_mut(tail.as_mut().get_mut());
            NonNullPtr::assign(&mut tail_link.next_ptr, item);
            NonNullPtr::assign_pin(&mut item_link.prev_ptr, tail);
        } else {
            NonNullPtr::assign(head_ptr, item);
        }
        NonNullPtr::assign(tail_ptr, item);
        self_.size.incr();
    }

    pub fn pop_front(self: Pin<&mut Self>) -> Option<NonNull<T>> {
        let self_ = Pin::into_inner(self);
        let head_ptr = &mut self_.link.next_ptr;
        if let Some(head) = head_ptr {
            let mut head = NonNull::from(head.as_mut().get_mut());
            let head_link = A::as_link_mut(unsafe { head.as_mut() });
            if let Some(next) = &mut head_link.next_ptr {
                let next_link = A::as_link_mut(next.as_mut().get_mut());
                NonNullPtr::assign_ptr(&mut next_link.prev_ptr, &mut head_link.prev_ptr);
                NonNullPtr::assign_pin(head_ptr, next);
            } else {
                self_.link.unlink();
            }
            head_link.unlink();
            self_.size.decr();
            Some(head)
        } else {
            None
        }
    }

    pub fn pop_back(self: Pin<&mut Self>) -> Option<NonNull<T>> {
        let self_ = Pin::into_inner(self);
        let tail_ptr = &mut self_.link.prev_ptr;
        if let Some(tail) = tail_ptr {
            let mut tail = NonNull::from(tail.as_mut().get_mut());
            let tail_link = A::as_link_mut(unsafe { tail.as_mut() });
            if let Some(prev) = &mut tail_link.prev_ptr {
                let prev_link = A::as_link_mut(prev.as_mut().get_mut());
                NonNullPtr::assign_ptr(&mut prev_link.next_ptr, &mut tail_link.next_ptr);
                NonNullPtr::assign_pin(tail_ptr, prev);
            } else {
                self_.link.unlink();
            }
            tail_link.unlink();
            self_.size.decr();
            Some(tail)
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

    pub fn back(self: Pin<&Self>) -> Option<Pin<&T>> {
        let self_ = Pin::into_inner(self);
        if let Some(last) = &self_.link.prev_ptr {
            Some(last.as_ref())
        } else {
            None
        }
    }

    pub fn back_mut(self: Pin<&mut Self>) -> Option<Pin<&mut T>> {
        let self_ = Pin::into_inner(self);
        if let Some(last) = &mut self_.link.prev_ptr {
            Some(last.as_mut())
        } else {
            None
        }
    }

    pub const fn iter(self: Pin<&Self>) -> Iter<T, A, P> {
        let self_ = Pin::into_inner(self);
        Iter {
            link: &self_.link,
            _marker: PhantomData,
        }
    }

    pub const fn iter_mut(self: Pin<&mut Self>) -> IterMut<T, A, P> {
        let self_ = Pin::into_inner(self);
        IterMut {
            link: &mut self_.link,
            _marker: PhantomData,
        }
    }

    pub fn into_iter(self: Pin<&mut Self>) -> IntoIter<T, A, P> {
        IntoIter { item: self }
    }

    pub fn is_empty(self: Pin<&Self>) -> bool {
        self.size.is_empty(self.iter())
    }

    pub fn len(self: Pin<&Self>) -> usize {
        self.size.len(self.iter())
    }

    pub fn contains(self: Pin<&Self>, data: &T) -> bool
    where
        T: PartialEq<T>,
    {
        for it in self.iter() {
            if data.eq(it.get_ref()) {
                return true;
            }
        }
        false
    }
}

impl<T, A, P> Default for DoublyLinkedList<T, A, P>
where
    A: Default,
{
    fn default() -> Self {
        Self::new(A::default())
    }
}

impl<T, A, P> Unpin for DoublyLinkedList<T, A, P> where T: Unpin {}

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

    impl PartialEq for X {
        fn eq(&self, other: &Self) -> bool {
            self.x == other.x
        }
    }

    #[derive(Default, Debug)]
    struct XLink;

    impl LinkAdapter<X> for XLink {
        type Link = Link<X>;

        fn as_link_ref(data: &X) -> &Self::Link {
            &data.link
        }

        fn as_link_mut(data: &mut X) -> &mut Self::Link {
            &mut data.link
        }
    }

    #[test]
    fn test_empty() {
        let mut lst = Box::pin(DoublyLinkedList::new(XLink));
        assert_eq!(lst.as_ref().is_empty(), true);
        assert_eq!(lst.as_ref().len(), 0);
        assert_eq!(lst.as_ref().front(), None);
        assert_eq!(lst.as_mut().front_mut(), None);
        assert_eq!(lst.as_ref().back(), None);
        assert_eq!(lst.as_mut().back_mut(), None);
    }

    #[test]
    fn test_push_pop() {
        let mut lst = Box::pin(DoublyLinkedList::new(XLink));
        lst.as_mut().push_front(X::new(1));
        // [1]
        assert_eq!(lst.as_ref().front().unwrap().x, 1);
        assert_eq!(lst.as_ref().back().unwrap().x, 1);
        assert_eq!(lst.as_ref().is_empty(), false);
        assert_eq!(lst.as_ref().len(), 1);

        lst.as_mut().push_front(X::new(2));
        // [2,1]
        assert_eq!(lst.as_ref().front().unwrap().x, 2);
        assert_eq!(lst.as_ref().back().unwrap().x, 1);
        assert_eq!(lst.as_ref().is_empty(), false);
        assert_eq!(lst.as_ref().len(), 2);

        lst.as_mut().push_front(X::new(3));
        // [3,2,1]
        assert_eq!(lst.as_ref().front().unwrap().x, 3);
        assert_eq!(lst.as_ref().back().unwrap().x, 1);
        assert_eq!(lst.as_ref().is_empty(), false);
        assert_eq!(lst.as_ref().len(), 3);

        let item = lst.as_mut().pop_front().unwrap();
        let item = unsafe { Box::from_raw(item.as_ptr()) };
        // [2,1]
        assert_eq!(item.link.is_linked(), false);
        assert_eq!(item.x, 3);
        assert_eq!(lst.as_ref().len(), 2);

        let item = lst.as_mut().pop_front().unwrap();
        let item = unsafe { Box::from_raw(item.as_ptr()) };
        // [1]
        assert_eq!(item.link.is_linked(), false);
        assert_eq!(item.x, 2);
        assert_eq!(lst.as_ref().len(), 1);

        lst.as_mut().push_front(X::new(4));
        // [4,1]
        assert_eq!(lst.as_ref().front().unwrap().x, 4);
        assert_eq!(lst.as_ref().back().unwrap().x, 1);
        assert_eq!(lst.as_ref().is_empty(), false);
        assert_eq!(lst.as_ref().len(), 2);

        lst.as_mut().push_front(X::new(5));
        // [5,4,1]
        assert_eq!(lst.as_ref().front().unwrap().x, 5);
        assert_eq!(lst.as_ref().back().unwrap().x, 1);
        assert_eq!(lst.as_ref().is_empty(), false);
        assert_eq!(lst.as_ref().len(), 3);

        let _ = lst.as_mut().pop_front().unwrap();
        let _ = lst.as_mut().pop_front().unwrap();
        let item = lst.as_mut().pop_front().unwrap();
        let item = unsafe { Box::from_raw(item.as_ptr()) };
        // []
        assert_eq!(item.link.is_linked(), false);
        assert_eq!(item.x, 1);
        assert_eq!(lst.as_ref().is_empty(), true);
        assert_eq!(lst.as_ref().len(), 0);
        assert_eq!(lst.as_mut().pop_front(), None);
    }

    #[test]
    fn test_push_pop_2() {
        let mut lst = Box::pin(DoublyLinkedList::new(XLink));
        lst.as_mut().push_back(X::new(1));
        // [1]
        assert_eq!(lst.as_ref().front().unwrap().x, 1);
        assert_eq!(lst.as_ref().back().unwrap().x, 1);
        assert_eq!(lst.as_ref().is_empty(), false);
        assert_eq!(lst.as_ref().len(), 1);

        lst.as_mut().push_back(X::new(2));
        // [1,2]
        assert_eq!(lst.as_ref().front().unwrap().x, 1);
        assert_eq!(lst.as_ref().back().unwrap().x, 2);
        assert_eq!(lst.as_ref().is_empty(), false);
        assert_eq!(lst.as_ref().len(), 2);

        lst.as_mut().push_front(X::new(3));
        // [3,1,2]
        assert_eq!(lst.as_ref().front().unwrap().x, 3);
        assert_eq!(lst.as_ref().back().unwrap().x, 2);
        assert_eq!(lst.as_ref().is_empty(), false);
        assert_eq!(lst.as_ref().len(), 3);

        lst.as_mut().push_back(X::new(4));
        // [3,1,2,4]
        assert_eq!(lst.as_ref().front().unwrap().x, 3);
        assert_eq!(lst.as_ref().back().unwrap().x, 4);
        assert_eq!(lst.as_ref().is_empty(), false);
        assert_eq!(lst.as_ref().len(), 4);

        let item = lst.as_mut().pop_back().unwrap();
        let item = unsafe { Box::from_raw(item.as_ptr()) };
        // [3,1,2]
        assert_eq!(item.link.is_linked(), false);
        assert_eq!(item.x, 4);
        assert_eq!(lst.as_ref().len(), 3);

        let item = lst.as_mut().pop_front().unwrap();
        let item = unsafe { Box::from_raw(item.as_ptr()) };
        // [1,2]
        assert_eq!(item.link.is_linked(), false);
        assert_eq!(item.x, 3);
        assert_eq!(lst.as_ref().len(), 2);

        lst.as_mut().push_back(X::new(4));
        // [1,2,4]
        assert_eq!(lst.as_ref().front().unwrap().x, 1);
        assert_eq!(lst.as_ref().back().unwrap().x, 4);
        assert_eq!(lst.as_ref().is_empty(), false);
        assert_eq!(lst.as_ref().len(), 3);

        lst.as_mut().push_front(X::new(5));
        // [5,1,2,4]
        assert_eq!(lst.as_ref().front().unwrap().x, 5);
        assert_eq!(lst.as_ref().back().unwrap().x, 4);
        assert_eq!(lst.as_ref().is_empty(), false);
        assert_eq!(lst.as_ref().len(), 4);

        let _ = lst.as_mut().pop_back().unwrap();
        let _ = lst.as_mut().pop_back().unwrap();
        let _ = lst.as_mut().pop_back().unwrap();
        let item = lst.as_mut().pop_front().unwrap();
        let item = unsafe { Box::from_raw(item.as_ptr()) };
        // []
        assert_eq!(item.link.is_linked(), false);
        assert_eq!(item.x, 5);
        assert_eq!(lst.as_ref().len(), 0);
        assert_eq!(lst.as_ref().is_empty(), true);
        assert_eq!(lst.as_mut().pop_front(), None);
    }
}
