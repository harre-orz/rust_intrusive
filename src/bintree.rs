use crate::ptr::{NonNullPtr, Pointer};
use crate::{LinkAdapter, OrdAdapter};
use std::cmp::Ordering;
use std::fmt;
use std::marker::PhantomData;
use std::pin::Pin;
use std::ptr::{self, NonNull};

pub struct Link<T, P = NonNull<T>> {
    top_ptr: Option<Pin<NonNullPtr<T, P>>>,
    left_ptr: Option<Pin<NonNullPtr<T, P>>>,
    right_ptr: Option<Pin<NonNullPtr<T, P>>>,
}

impl<T, P> Link<T, P> {
    pub const fn new() -> Self {
        Self {
            top_ptr: None,
            left_ptr: None,
            right_ptr: None,
        }
    }

    pub const fn is_linked(&self) -> bool {
        self.left_ptr.is_some() || self.right_ptr.is_some()
    }

    pub unsafe fn unlink(&mut self) {
        self.top_ptr = None;
        self.left_ptr = None;
        self.right_ptr = None;
    }
}

impl<T, P> Link<T, P>
where
    T: Unpin,
    P: Pointer<T>,
{
    fn is_left(&self, data: &T) -> bool {
        if let Some(left) = &self.left_ptr {
            let left: *const T = left.as_ref().get_ref();
            left.eq(&(data as *const T))
        } else {
            false
        }
    }

    fn is_right(&self, data: &T) -> bool {
        if let Some(right) = &self.right_ptr {
            let right: *const T = right.as_ref().get_ref();
            right.eq(&(data as *const T))
        } else {
            false
        }
    }
}

impl<T, P> Default for Link<T, P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, P> Unpin for Link<T, P> where T: Unpin {}

impl<T, P> fmt::Debug for Link<T, P>
where
    T: Unpin + fmt::Debug,
    P: Pointer<T>,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Link {{ ")?;

        // left
        if let Some(item) = &self.left_ptr {
            write!(fmt, "L = ({:p}) {:?}, ", item.as_ref(), item.as_ref())?;
        } else {
            write!(fmt, "L = None, ")?;
        }

        // right
        if let Some(item) = &self.right_ptr {
            write!(fmt, "R = ({:p}) {:?} ", item.as_ref(), item.as_ref())?;
        } else {
            write!(fmt, "R = None, ")?;
        }

        write!(fmt, " }}")
    }
}

pub struct Iter<'a, T, A, P> {
    link: *const Link<T, P>,
    _marker: PhantomData<&'a A>,
}

impl<'a, T, A, P> Iter<'a, T, A, P>
where
    T: Unpin + 'a,
    P: Pointer<T> + 'a,
    A: LinkAdapter<T, Link = Link<T, P>>,
{
    fn left_end_leaf(ptr: &Pin<NonNullPtr<T, P>>) -> Pin<&T> {
        let mut item_ptr = ptr.as_ref().get_ref();
        loop {
            let item = unsafe { &*item_ptr };
            let link = A::as_link_ref(item);
            if let Some(next) = &link.left_ptr {
                item_ptr = next.as_ref().get_ref();
            } else {
                return Pin::new(item);
            }
        }
    }
    // fn leftest_ptr(ptr: &Option<Pin<NonNullPtr<T, P>>>) -> *const Pin<NonNullPtr<T, P>> {
    //     let mut item_ptr = if let Some(item) = ptr {
    //         item as *const Pin<NonNullPtr<T, P>>
    //     } else {
    //         return ptr::null();
    //     };
    //     loop {
    //         let link = A::as_link_ref(unsafe { &*item_ptr });
    //         if let Some(item) = &link.left_ptr {
    //             item_ptr = item
    //         } else {
    //             return item_ptr;
    //         }
    //     }
    // }
    //
    // fn rightest_ptr(ptr: &Option<Pin<NonNullPtr<T, P>>>) -> *const Pin<NonNullPtr<T, P>> {
    //     let mut item_ptr = if let Some(item) = ptr {
    //         item as *const Pin<NonNullPtr<T, P>>
    //     } else {
    //         return ptr::null();
    //     };
    //     loop {
    //         let link = A::as_link_ref(unsafe { &*item_ptr });
    //         if let Some(item) = &link.right_ptr {
    //             item_ptr = item
    //         } else {
    //             return item_ptr;
    //         }
    //     }
    // }
}

impl<'a, T, A, P> Iterator for Iter<'a, T, A, P>
where
    T: Unpin + 'a,
    P: Pointer<T> + 'a,
    A: LinkAdapter<T, Link = Link<T, P>>,
{
    type Item = Pin<&'a T>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl<'a, T, A, P> DoubleEndedIterator for Iter<'a, T, A, P>
where
    T: Unpin + 'a,
    P: Pointer<T> + 'a,
    A: LinkAdapter<T, Link = Link<T, P>>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        None
    }
}

pub struct IterMut<'a, T, A, P> {
    link: *mut Link<T, P>,
    _marker: PhantomData<&'a A>,
}

impl<'a, T, A, P> IterMut<'a, T, A, P>
where
    T: Unpin + 'a,
    P: Pointer<T> + 'a,
    A: LinkAdapter<T, Link = Link<T, P>>,
{
    fn leftest_ptr(ptr: &mut Option<Pin<NonNullPtr<T, P>>>) -> *mut Pin<NonNullPtr<T, P>> {
        let mut item_ptr = if let Some(item) = ptr {
            item as *mut Pin<NonNullPtr<T, P>>
        } else {
            return ptr::null_mut();
        };
        loop {
            let link = A::as_link_mut(unsafe { &mut *item_ptr });
            if let Some(item) = &mut link.left_ptr {
                item_ptr = item
            } else {
                return item_ptr;
            }
        }
    }

    fn rightest_ptr(ptr: &mut Option<Pin<NonNullPtr<T, P>>>) -> *mut Pin<NonNullPtr<T, P>> {
        let mut item_ptr = if let Some(item) = ptr {
            item as *mut Pin<NonNullPtr<T, P>>
        } else {
            return ptr::null_mut();
        };
        loop {
            let link = A::as_link_mut(unsafe { &mut *item_ptr });
            if let Some(item) = &mut link.right_ptr {
                item_ptr = item
            } else {
                return item_ptr;
            }
        }
    }
}

impl<'a, T, A, P> Iterator for IterMut<'a, T, A, P>
where
    T: Unpin + 'a,
    P: Pointer<T> + 'a,
    A: LinkAdapter<T, Link = Link<T, P>>,
{
    type Item = Pin<&'a mut T>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl<'a, T, A, P> DoubleEndedIterator for IterMut<'a, T, A, P>
where
    T: Unpin + 'a,
    P: Pointer<T> + 'a,
    A: LinkAdapter<T, Link = Link<T, P>>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        None
    }
}

pub struct BinTree<T, A, P = NonNull<T>> {
    link: Link<T, P>,
    size: A,
}

impl<T, A, P> BinTree<T, A, P> {
    pub const fn new(adapter: A) -> Self {
        Self {
            link: Link::new(),
            size: adapter,
        }
    }
}

impl<T, A, P> BinTree<T, A, P>
where
    T: Unpin,
    P: Pointer<T>,
    A: LinkAdapter<T, Link = Link<T, P>> + OrdAdapter<T>,
{
    pub fn get(self: Pin<&Self>, key: &T) -> Option<Pin<&T>> {
        let self_ = Pin::into_inner(self);
        let mut node_ptr = &self_.link.top_ptr as *const Option<Pin<NonNullPtr<T, P>>>;
        loop {
            if let Some(node) = unsafe { &*node_ptr } {
                match A::cmp(node.as_ref().get_ref(), key) {
                    Ordering::Equal => return Some(Pin::new(node)),
                    Ordering::Less => {
                        let link: &Link<T, P> = A::as_link_ref(node);
                        node_ptr = &link.left_ptr
                    }
                    Ordering::Greater => {
                        let link: &Link<T, P> = A::as_link_ref(node);
                        node_ptr = &link.right_ptr
                    }
                }
            } else {
                return None;
            }
        }
    }

    pub fn get_mut(self: Pin<&mut Self>, key: &T) -> Option<Pin<&mut T>> {
        let self_ = Pin::into_inner(self);
        let mut node_ptr: *mut Option<Pin<NonNullPtr<T, P>>> = &mut self_.link.top_ptr;
        loop {
            if let Some(node) = unsafe { &mut *node_ptr } {
                match A::cmp(node.as_ref().get_ref(), key) {
                    Ordering::Equal => return Some(Pin::new(node)),
                    Ordering::Less => {
                        let link: &mut Link<T, P> = A::as_link_mut(node);
                        node_ptr = &mut link.left_ptr
                    }
                    Ordering::Greater => {
                        let link: &mut Link<T, P> = A::as_link_mut(node);
                        node_ptr = &mut link.right_ptr
                    }
                }
            } else {
                return None;
            }
        }
    }

    pub fn pop_front(self: Pin<&mut Self>) -> Option<NonNull<T>> {
        None
    }

    pub fn pop_back(self: Pin<&mut Self>) -> Option<NonNull<T>> {
        None
    }

    pub fn front(self: Pin<&Self>) -> Option<&T> {
        None
    }

    pub fn front_mut(self: Pin<&mut Self>) -> Option<&mut T> {
        None
    }

    pub fn back(self: Pin<&Self>) -> Option<&T> {
        None
    }

    pub fn back_mut(self: Pin<&mut Self>) -> Option<&mut T> {
        None
    }

    pub fn insert(self: Pin<&mut Self>, mut item: NonNull<T>) -> Option<NonNull<T>> {
        let item_link = A::as_link_mut(unsafe { item.as_mut() });
        debug_assert_eq!(item_link.is_linked(), false);

        let self_ = Pin::into_inner(self);
        self_.size.incr();
        let mut node = if let Some(node) = &mut self_.link.top_ptr {
            node as *mut Pin<NonNullPtr<T, P>>
        } else {
            NonNullPtr::assign(&mut self_.link.top_ptr, item);
            NonNullPtr::assign(&mut self_.link.left_ptr, item);
            NonNullPtr::assign(&mut self_.link.right_ptr, item);
            return None;
        };
        let mut is_left_end = true;
        let mut is_right_end = true;
        loop {
            let cmp = A::cmp(unsafe { item.as_ref() }, unsafe { &*node }.as_ref().get_ref());
            match cmp {
                Ordering::Less => {
                    // item < node
                    let link = A::as_link_mut(unsafe { &mut *node }.as_mut().get_mut());
                    if let Some(node_) = &mut link.left_ptr {
                        node = node_;
                    } else {
                        NonNullPtr::assign_pin(&mut item_link.top_ptr, unsafe { &mut *node });
                        NonNullPtr::assign(&mut link.left_ptr, item);
                        if is_left_end {
                            NonNullPtr::assign(&mut self_.link.left_ptr, item);
                        }
                        return None
                    }
                    is_right_end = false;
                },
                Ordering::Greater => {
                    // item > node
                    let link = A::as_link_mut(unsafe { &mut *node }.as_mut().get_mut());
                    if let Some(node_) = &mut link.right_ptr {
                        node = node_;
                    } else {
                        NonNullPtr::assign_pin(&mut item_link.top_ptr, unsafe { &mut *node });
                        NonNullPtr::assign(&mut link.right_ptr, item);
                        if is_right_end {
                            NonNullPtr::assign(&mut self_.link.right_ptr, item);
                        }
                        return None
                    }
                    is_left_end = false;
                },
                _ => return Some(item),
            }
        }
    }

    pub fn remove(self: Pin<&mut Self>, _data: &T) -> Option<NonNull<T>> {
        None
    }

    pub fn iter(self: Pin<&Self>) -> Iter<T, A, P> {
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

    pub fn is_empty(self: Pin<&Self>) -> bool {
        self.size.is_empty(self.iter())
    }

    pub fn len(self: Pin<&Self>) -> usize {
        self.size.len(self.iter())
    }
}

impl<T, A, P> Default for BinTree<T, A, P>
where
    A: Default,
{
    fn default() -> Self {
        Self::new(A::default())
    }
}

impl<T, A, P> Unpin for BinTree<T, A, P> where T: Unpin {}

impl<T, A, P> fmt::Debug for BinTree<T, A, P>
where
    T: Unpin + fmt::Debug,
    P: Pointer<T>,
    A: fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if let Some(root) = &self.link.top_ptr {
            write!(
                fmt,
                "BinTree {{ size: {:?}, root: ({:p}) {:?} }}",
                self.size,
                root.as_ref(),
                root.as_ref()
            )
        } else {
            write!(fmt, "BinTree {{ size: {:?}, root: None }}", self.size)
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

    #[derive(Debug, Default)]
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

    impl OrdAdapter<X> for XLink {
        fn cmp(left: &X, right: &X) -> Ordering {
            left.x.cmp(&right.x)
        }
    }

    #[test]
    fn test() {
        let mut tree = Box::pin(BinTree::new(XLink));
        assert_eq!(tree.as_ref().len(), 0);

        tree.as_mut().insert(X::new(1));
        assert_eq!(tree.as_ref().len(), 1);
        {
            println!("{:?}", tree);
            let mut it = tree.as_ref().iter();
            assert_eq!(it.next().unwrap().x, 1);
        }
        tree.as_mut().insert(X::new(2));
        assert_eq!(tree.as_ref().len(), 2);
        tree.as_mut().insert(X::new(0));
        assert_eq!(tree.as_ref().len(), 3);
        {
            println!("{:?}", tree);
            let mut it = tree.as_ref().iter();
            assert_eq!(it.next().unwrap().x, 0);
            assert_eq!(it.next().unwrap().x, 1);
            assert_eq!(it.next().unwrap().x, 2);
            assert_eq!(it.next().is_none(), true);
        }
    }
}
