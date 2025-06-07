use crate::adapter::{LinkAdapter, Size};
use crate::ptr::{NonNullPtr, Pointer};
use std::cmp;
use std::marker::PhantomData;
use std::pin::Pin;
use std::ptr::NonNull;
use std::fmt;
use std::fmt::Formatter;

pub struct Link<T, P = NonNull<T>> {
    next_ptr: Option<Pin<NonNullPtr<T, P>>>,
}

impl<T, P> Link<T, P> {
    pub const fn new() -> Self {
        Self { next_ptr: None }
    }

    pub const fn is_linked(&self) -> bool {
        self.next_ptr.is_some()
    }

    fn unlink(&mut self) {
        self.next_ptr = None;
    }
}

impl<T, P> Default for Link<T, P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, P> Unpin for Link<T, P> where T: Unpin {}

impl<T, P> cmp::PartialEq for Link<T, P> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl<T, P> cmp::Eq for Link<T, P> {}

impl<T, P> cmp::PartialOrd for Link<T, P> {
    fn partial_cmp(&self, _: &Self) -> Option<cmp::Ordering> {
        Some(cmp::Ordering::Equal)
    }
}

impl<T, P> cmp::Ord for Link<T, P> {
    fn cmp(&self, _: &Self) -> cmp::Ordering {
        cmp::Ordering::Equal
    }
}

impl<T, P> fmt::Debug for Link<T, P>
where
    P: fmt::Debug,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{{ next: ")?;
        if let Some(next) = &self.next_ptr {
            write!(f, "{:?}", next)?;
        } else {
            write!(f, "0x0")?;
        }
        write!(f, " }}")
    }
}

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

pub struct IntoIter<'a, T, A, P>
where
    A: LinkAdapter<T>,
{
    item: Pin<&'a mut SinglyLinkedList<T, A, P>>,
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

#[derive(Debug)]
pub struct SinglyLinkedList<T, A, P = NonNull<T>>
where
    A: LinkAdapter<T>,
{
    size: A::Size,
    link: Link<T, P>,
}

impl<T, A, P> SinglyLinkedList<T, A, P>
where
    A: LinkAdapter<T>,
{
    pub fn new(_: A) -> Self {
        Self {
            link: Link::new(),
            size: Default::default(),
        }
    }
}

impl<T, A, P> SinglyLinkedList<T, A, P>
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
        if let Some(head) = head_ptr {
            NonNullPtr::assign_pin(&mut item_link.next_ptr, head);
        }
        NonNullPtr::assign(head_ptr, item);
        self_.size.incr();
    }

    pub fn pop_front(self: Pin<&mut Self>) -> Option<NonNull<T>> {
        let self_ = Pin::into_inner(self);
        let head_ptr = &mut self_.link.next_ptr;
        if let Some(head) = head_ptr {
            let mut head = NonNull::from(head.as_mut().get_mut());
            let head_link = A::as_link_mut(unsafe { head.as_mut() });
            if let Some(next) = &mut head_link.next_ptr {
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

    pub const fn iter(self: Pin<&Self>) -> Iter<T, A, P> {
        let self_ = Pin::into_inner(self);
        Iter {
            link: &self_.link,
            _marker: PhantomData,
        }
    }

    pub fn iter_mut(self: Pin<&mut Self>) -> IterMut<T, A, P> {
        let self_ = Pin::into_inner(self);
        IterMut {
            link: &mut self_.link,
            _marker: PhantomData,
        }
    }

    pub const fn into_iter(self: Pin<&mut Self>) -> IntoIter<T, A, P> {
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

    pub fn append(mut self: Pin<&mut Self>, other: Pin<&mut Self>) {
        let vec: Vec<_> = other.into_iter().collect();
        for data in vec.into_iter().rev() {
            self.as_mut().push_front(data);
        }
    }
}

impl<T, A, P> Default for SinglyLinkedList<T, A, P>
where
    A: LinkAdapter<T> + Default,
{
    fn default() -> Self {
        Self::new(A::default())
    }
}

impl<T, A, P> Unpin for SinglyLinkedList<T, A, P>
where
    T: Unpin,
    A: LinkAdapter<T>,
{
}

#[cfg(test)]
mod test {
    use std::fmt::Formatter;
    use super::*;
    use crate::adapter::NumerateSize;

    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    struct X {
        data: i32,
        link: Link<Self>,
    }

    impl X {
        fn new(data: i32) -> NonNull<Self> {
            let ptr = Box::new(X {
                data: data,
                link: Link::new(),
            });
            let ptr = Box::into_raw(ptr);
            NonNull::new(ptr).unwrap()
        }

        fn from(item: Option<NonNull<Self>>) -> Option<Box<Self>> {
            if let Some(data) = item {
                let ptr = unsafe { Box::from_raw(data.as_ptr()) };
                assert_eq!(ptr.link.is_linked(), false);
                Some(ptr)
            } else {
                None
            }
        }
    }

    impl fmt::Debug for X {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "X ({:p}) {{ data: {:?}, link: {:?} }}", self, self.data, self.link)
        }
    }

    #[derive(Debug)]
    struct XLink;

    impl LinkAdapter<X> for XLink {
        type Link = Link<X>;
        type Size = NumerateSize;

        fn as_link_ref(data: &X) -> &Self::Link {
            &data.link
        }

        fn as_link_mut(data: &mut X) -> &mut Self::Link {
            &mut data.link
        }
    }

    #[test]
    fn test_empty() {
        let mut lst = Box::pin(SinglyLinkedList::new(XLink));
        assert_eq!(lst.as_ref().is_empty(), true);
        assert_eq!(lst.as_ref().len(), 0);
        assert_eq!(lst.as_ref().front(), None);
        assert_eq!(lst.as_mut().front_mut(), None);
    }

    #[test]
    fn test_push_pop() {
        let mut lst = Box::pin(SinglyLinkedList::new(XLink));
        lst.as_mut().push_front(X::new(1));
        let item = lst.as_ref().front().unwrap();
        // [1]
        // println!("{:?}", lst);
        // println!("{:?}", item);
        assert_eq!(item.data, 1);
        assert_eq!(lst.as_ref().is_empty(), false);
        assert_eq!(lst.as_ref().len(), 1);

        lst.as_mut().push_front(X::new(2));
        let item = lst.as_ref().front().unwrap();
        // [2,1]
        // println!("{:?}", lst);
        // println!("{:?}", item);
        assert_eq!(item.data, 2);
        assert_eq!(lst.as_ref().is_empty(), false);
        assert_eq!(lst.as_ref().len(), 2);

        lst.as_mut().push_front(X::new(3));
        let item = lst.as_ref().front().unwrap();
        // [3,2,1]
        assert_eq!(item.data, 3);
        assert_eq!(lst.as_ref().is_empty(), false);
        assert_eq!(lst.as_ref().len(), 3);

        let item = X::from(lst.as_mut().pop_front()).unwrap();
        // [2,1]
        assert_eq!(item.link.is_linked(), false);
        assert_eq!(item.data, 3);
        assert_eq!(lst.as_ref().len(), 2);

        let item = X::from(lst.as_mut().pop_front()).unwrap();
        // [1]
        assert_eq!(item.link.is_linked(), false);
        assert_eq!(item.data, 2);
        assert_eq!(lst.as_ref().len(), 1);

        lst.as_mut().push_front(X::new(4));
        let item = lst.as_ref().front().unwrap();
        // [4,1]
        assert_eq!(item.data, 4);
        assert_eq!(lst.as_ref().is_empty(), false);
        assert_eq!(lst.as_ref().len(), 2);

        lst.as_mut().push_front(X::new(5));
        let item = lst.as_ref().front().unwrap();
        // [5,4,1]
        assert_eq!(item.data, 5);
        assert_eq!(lst.as_ref().is_empty(), false);
        assert_eq!(lst.as_ref().len(), 3);

        let _ = lst.as_mut().pop_front();
        let _ = lst.as_mut().pop_front();
        let item = X::from(lst.as_mut().pop_front()).unwrap();
        // []
        assert_eq!(item.link.is_linked(), false);
        assert_eq!(item.data, 1);
        assert_eq!(lst.as_ref().len(), 0);
        assert_eq!(lst.as_mut().pop_front(), None);
    }
}
