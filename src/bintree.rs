use crate::adapter::{LinkAdapter, Size};
use crate::ptr::{NonNullPtr, Pointer};
use std::cmp;
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
        self.top_ptr.is_some() || self.left_ptr.is_some() || self.right_ptr.is_some()
    }

    fn unlink(&mut self) {
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
    fn first_left<A>(&self) -> Option<&Pin<NonNullPtr<T, P>>>
    where
        A: LinkAdapter<T, Link = Link<T, P>>,
    {
        if let Some(left) = &self.left_ptr {
            let left_link = A::link_ref(left.as_ref().get_ref());
            if let Some(top) = &left_link.top_ptr {
                let top_link = A::link_ref(top.as_ref().get_ref());
                if ptr::addr_eq(self, top_link) {
                    None
                } else {
                    Some(left)
                }
            } else if let Some(top) = &self.top_ptr {
                let top_link = A::link_ref(top.as_ref().get_ref());
                if ptr::addr_eq(self, top_link) {
                    None
                } else {
                    Some(left)
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    fn first_left_mut<A>(&mut self) -> Option<&mut Pin<NonNullPtr<T, P>>>
        where
            A: LinkAdapter<T, Link = Link<T, P>>,
    {
        let self_ = self as *const Self;
        if let Some(left) = &mut self.left_ptr {
            let left_link = A::link_mut(left.as_mut().get_mut());
            if let Some(top) = &mut left_link.top_ptr {
                let top_link = A::link_mut(top.as_mut().get_mut());
                if ptr::addr_eq(self_, top_link) {
                    None
                } else {
                    Some(left)
                }
            } else if let Some(top) = &mut self.top_ptr {
                let top_link = A::link_mut(top.as_mut().get_mut());
                if ptr::addr_eq(self_, top_link) {
                    None
                } else {
                    Some(left)
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    fn is_left<A>(&self, link: &Self) -> bool
    where
        A: LinkAdapter<T, Link = Link<T, P>>,
    {
        if let Some(left) = &self.left_ptr {
            ptr::addr_eq(link, A::link_ref(left))
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ top: ")?;
        if let Some(top) = &self.top_ptr {
            write!(f, "{:?}", top)?;
        } else {
            write!(f, "0x0")?;
        }
        write!(f, ", left: ")?;
        if let Some(left) = &self.left_ptr {
            write!(f, "{:?}", left)?;
        } else {
            write!(f, "0x0")?;
        }
        write!(f, ", right: ")?;
        if let Some(right) = &self.right_ptr {
            write!(f, "{:?}", right)?;
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
        if let Some(node) = link.first_left::<A>() {
            self.link = A::link_ref(node.as_ref().get_ref());
            Some(node.as_ref())
        } else if let Some(node) = &link.right_ptr {
            let mut node = node as *const Pin<NonNullPtr<T, P>>;
            loop {
                let node_link = A::link_ref(unsafe { &*node }.as_ref().get_ref());
                if let Some(left) = &node_link.left_ptr {
                    node = left;
                } else {
                    self.link = node_link;
                    return Some(unsafe { &*node }.as_ref());
                }
            }
        } else if let Some(node) = &link.top_ptr {
            let mut node = node as *const Pin<NonNullPtr<T, P>>;
            let mut link = link as *const Link<T, P>;
            loop {
                let node_link = A::link_ref(unsafe { &*node }.as_ref().get_ref());
                if node_link.is_left::<A>(unsafe { &*link }) {
                    self.link = node_link;
                    return Some(unsafe { &*node }.as_ref());
                } else if let Some(top) = &node_link.top_ptr {
                    link = node_link;
                    node = top;
                } else {
                    return None;
                }
            }
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
        todo!()
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
        let link = self.link;
        if let Some(node) = unsafe { &mut *link }.first_left_mut::<A>() {
            self.link = A::link_mut(node.as_mut().get_mut());
            Some(node.as_mut())
        } else if let Some(node) = &mut unsafe { &mut *link }.right_ptr {
            let mut node = node as *mut Pin<NonNullPtr<T, P>>;
            loop {
                let node_link = A::link_mut(unsafe { &mut *node }.as_mut().get_mut());
                if let Some(left) = &mut node_link.left_ptr {
                    node = left;
                } else {
                    self.link = node_link;
                    return Some(unsafe { &mut *node }.as_mut());
                }
            }
        } else if let Some(node) = &mut unsafe { &mut *link }.top_ptr {
            let mut node = node as *mut Pin<NonNullPtr<T, P>>;
            let mut link = link as *mut Link<T, P>;
            loop {
                let node_link = A::link_mut(unsafe { &mut *node }.as_mut().get_mut()) as *mut Link<T, P>;
                if unsafe { &mut *node_link }.is_left::<A>(unsafe { &mut *link }) {
                    self.link = node_link;
                    return Some(unsafe { &mut *node }.as_mut());
                } else if let Some(top) = &mut unsafe { &mut *node_link }.top_ptr {
                    link = node_link;
                    node = top;
                } else {
                    return None;
                }
            }
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
        todo!()
    }
}

#[derive(Debug)]
pub struct BinTree<T, A, P = NonNull<T>>
where
    A: LinkAdapter<T>,
{
    size: A::Size,
    link: Link<T, P>,
}

impl<T, A, P> BinTree<T, A, P>
where
    A: LinkAdapter<T>,
{
    pub fn new(_: A) -> Self {
        Self {
            size: Default::default(),
            link: Link::new(),
        }
    }
}

impl<T, A, P> BinTree<T, A, P>
where
    T: Unpin + cmp::Ord,
    P: Pointer<T>,
    A: LinkAdapter<T, Link = Link<T, P>>,
{
    pub fn get(self: Pin<&Self>, key: &T) -> Option<Pin<&T>> {
        let self_ = Pin::into_inner(self);
        let mut node_ptr = &self_.link.top_ptr as *const Option<Pin<NonNullPtr<T, P>>>;
        loop {
            if let Some(node) = unsafe { &*node_ptr } {
                match node.as_ref().get_ref().cmp(key) {
                    cmp::Ordering::Equal => return Some(Pin::new(node)),
                    cmp::Ordering::Less => {
                        let link: &Link<T, P> = A::link_ref(node);
                        node_ptr = &link.left_ptr
                    }
                    cmp::Ordering::Greater => {
                        let link: &Link<T, P> = A::link_ref(node);
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
                match node.as_ref().get_ref().cmp(key) {
                    cmp::Ordering::Equal => return Some(Pin::new(node)),
                    cmp::Ordering::Less => {
                        let link: &mut Link<T, P> = A::link_mut(node);
                        node_ptr = &mut link.left_ptr
                    }
                    cmp::Ordering::Greater => {
                        let link: &mut Link<T, P> = A::link_mut(node);
                        node_ptr = &mut link.right_ptr
                    }
                }
            } else {
                return None;
            }
        }
    }

    pub fn insert(self: Pin<&mut Self>, mut item: NonNull<T>) -> Option<NonNull<T>> {
        let item_link = A::link_mut(unsafe { item.as_mut() });
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
            let cmp = unsafe { item.as_ref().cmp((&*node).as_ref().get_ref()) };
            match cmp {
                cmp::Ordering::Less => {
                    // item < node
                    let link = A::link_mut(unsafe { &mut *node }.as_mut().get_mut());
                    if let Some(left) = &mut link.left_ptr {
                        node = left;
                    } else {
                        NonNullPtr::assign_pin(&mut item_link.top_ptr, unsafe { &mut *node });
                        NonNullPtr::assign(&mut link.left_ptr, item);
                        if is_left_end {
                            NonNullPtr::assign(&mut self_.link.left_ptr, item);
                        }
                        return None;
                    }
                    is_right_end = false;
                }
                cmp::Ordering::Greater => {
                    // item > node
                    let link = A::link_mut(unsafe { &mut *node }.as_mut().get_mut());
                    if let Some(right) = &mut link.right_ptr {
                        node = right;
                    } else {
                        NonNullPtr::assign_pin(&mut item_link.top_ptr, unsafe { &mut *node });
                        NonNullPtr::assign(&mut link.right_ptr, item);
                        if is_right_end {
                            NonNullPtr::assign(&mut self_.link.right_ptr, item);
                        }
                        return None;
                    }
                    is_left_end = false;
                }
                _ => return Some(item),
            }
        }
    }

    pub fn pop_front(self: Pin<&mut Self>) -> Option<NonNull<T>> {
        let self_ = Pin::into_inner(self);
        if let Some(node) = &mut self_.link.left_ptr {
            let mut node = NonNull::from(node.as_mut().get_mut());
            let node_link = A::link_mut(unsafe { node.as_mut() });
            if let Some(right) = &node_link.right_ptr {
                todo!()
            } else if let Some(top) = &mut node_link.top_ptr {
                NonNullPtr::assign_pin(&mut self_.link.left_ptr, top);
                let top_link = A::link_mut(unsafe { top.as_mut().get_mut() });
                top_link.left_ptr = None;
            } else {
                self_.link.unlink();
            }
            node_link.unlink();
            self_.size.decr();
            Some(node)
        } else {
            None
        }
    }

    pub fn pop_back(self: Pin<&mut Self>) -> Option<NonNull<T>> {
        let self_ = Pin::into_inner(self);
        if let Some(node) = &mut self_.link.right_ptr {
            let mut node = NonNull::from(node.as_mut().get_mut());
            let node_link = A::link_mut(unsafe { node.as_mut() });
            if let Some(left) = &node_link.left_ptr {
                todo!()
            } else if let Some(top) = &mut node_link.top_ptr {
                NonNullPtr::assign_pin(&mut self_.link.right_ptr, top);
                let top_link = A::link_mut(unsafe { top.as_mut().get_mut() });
                top_link.right_ptr = None;
            } else {
                self_.link.unlink();
            }
            node_link.unlink();
            self_.size.decr();
            Some(node)
        } else {
            None
        }
    }

    pub fn front(self: Pin<&Self>) -> Option<&T> {
        let self_ = Pin::into_inner(self);
        if let Some(node) = &self_.link.left_ptr {
            Some(node)
        } else {
            None
        }
    }

    pub fn front_mut(self: Pin<&mut Self>) -> Option<&mut T> {
        let self_ = Pin::into_inner(self);
        if let Some(node) = &mut self_.link.left_ptr {
            Some(node)
        } else {
            None
        }
    }

    pub fn back(self: Pin<&Self>) -> Option<&T> {
        let self_ = Pin::into_inner(self);
        if let Some(node) = &self_.link.right_ptr {
            Some(node)
        } else {
            None
        }
    }

    pub fn back_mut(self: Pin<&mut Self>) -> Option<&mut T> {
        let self_ = Pin::into_inner(self);
        if let Some(node) = &mut self_.link.right_ptr {
            Some(node)
        } else {
            None
        }
    }

    pub fn remove(self: Pin<&mut Self>, _data: &T) -> Option<NonNull<T>> {
        todo!()
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

    pub fn len(self: Pin<&Self>) -> usize {
        self.size.len(self.iter())
    }

    pub fn is_empty(self: Pin<&Self>) -> bool {
        self.size.is_empty(self.iter())
    }
}

impl<T, A, P> Default for BinTree<T, A, P>
where
    A: LinkAdapter<T> + Default,
{
    fn default() -> Self {
        Self::new(A::default())
    }
}

impl<T, A, P> Unpin for BinTree<T, A, P>
where
    T: Unpin,
    A: LinkAdapter<T>,
{
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::adapter::NumerateSize;
    use std::fmt::Formatter;

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

        fn from(data: Option<NonNull<Self>>) -> Option<Box<Self>> {
            if let Some(data) = data {
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
            write!(
                f,
                "X ({:p}) {{ data: {:?}, link: {:?} }}",
                self, self.data, self.link
            )
        }
    }

    #[derive(Debug)]
    struct XLink;

    impl LinkAdapter<X> for XLink {
        type Link = Link<X>;
        type Size = NumerateSize;

        fn link_ref(data: &X) -> &Self::Link {
            &data.link
        }

        fn link_mut(data: &mut X) -> &mut Self::Link {
            &mut data.link
        }
    }

    #[test]
    fn test_empty() {
        let tree = Box::pin(BinTree::new(XLink));
        // []
        assert_eq!(tree.as_ref().len(), 0);
        assert_eq!(tree.as_ref().is_empty(), true);
    }

    #[test]
    fn test_insert_1() {
        let mut tree = Box::pin(BinTree::new(XLink));
        tree.as_mut().insert(X::new(1));
        // [1]
        let mut it = tree.as_ref().iter();
        assert_eq!(it.next().unwrap().data, 1);
        assert_eq!(it.next(), None);
        assert_eq!(tree.as_ref().len(), 1);
        assert_eq!(tree.as_ref().is_empty(), false);
        assert_eq!(tree.as_ref().front().unwrap().data, 1);
        assert_eq!(tree.as_mut().front_mut().unwrap().data, 1);
        assert_eq!(tree.as_ref().back().unwrap().data, 1);
        assert_eq!(tree.as_mut().back_mut().unwrap().data, 1);

        let item = X::from(tree.as_mut().pop_front()).unwrap();
        // []
        assert_eq!(item.data, 1);
        assert_eq!(tree.as_ref().len(), 0);
        assert_eq!(tree.as_ref().is_empty(), true);

        tree.as_mut().insert(X::new(2));
        // [1]
        assert_eq!(tree.as_ref().len(), 1);
        assert_eq!(tree.as_ref().is_empty(), false);
        assert_eq!(tree.as_ref().front().unwrap().data, 2);
        assert_eq!(tree.as_mut().front_mut().unwrap().data, 2);
        assert_eq!(tree.as_ref().back().unwrap().data, 2);
        assert_eq!(tree.as_mut().back_mut().unwrap().data, 2);

        let item = X::from(tree.as_mut().pop_back()).unwrap();
        // []
        assert_eq!(item.data, 2);
        assert_eq!(tree.as_ref().len(), 0);
        assert_eq!(tree.as_ref().is_empty(), true);
    }

    #[test]
    fn test_insert_2() {
        let mut tree = Box::pin(BinTree::new(XLink));
        tree.as_mut().insert(X::new(1));
        tree.as_mut().insert(X::new(2));
        tree.as_mut().insert(X::new(3));

        // let mut it = tree.as_ref().iter();
        // println!("{:?}", tree);
        // println!("{:?}", it.next().unwrap());
        // println!("{:?}", it.next().unwrap());
        // println!("{:?}", it.next().unwrap());

        // [1-2-3]
        let mut it = tree.as_ref().iter();
        assert_eq!(it.next().unwrap().data, 1);
        assert_eq!(it.next().unwrap().data, 2);
        assert_eq!(it.next().unwrap().data, 3);
        assert_eq!(it.next(), None);
        assert_eq!(tree.as_ref().front().unwrap().data, 1);
        assert_eq!(tree.as_mut().front_mut().unwrap().data, 1);
        assert_eq!(tree.as_ref().back().unwrap().data, 3);
        assert_eq!(tree.as_mut().back_mut().unwrap().data, 3);
        assert_eq!(tree.as_ref().len(), 3);
        assert_eq!(tree.as_ref().is_empty(), false);

        let item = X::from(tree.as_mut().pop_back()).unwrap();
        // [1-2]
        let mut it = tree.as_ref().iter();
        assert_eq!(it.next().unwrap().data, 1);
        assert_eq!(it.next().unwrap().data, 2);
        assert_eq!(it.next(), None);
        assert_eq!(item.data, 3);
        assert_eq!(tree.as_ref().front().unwrap().data, 1);
        assert_eq!(tree.as_mut().front_mut().unwrap().data, 1);
        assert_eq!(tree.as_ref().back().unwrap().data, 2);
        assert_eq!(tree.as_mut().back_mut().unwrap().data, 2);
        assert_eq!(tree.as_ref().len(), 2);
        assert_eq!(tree.as_ref().is_empty(), false);

        let item = X::from(tree.as_mut().pop_back()).unwrap();
        // [1]
        let mut it = tree.as_ref().iter();
        assert_eq!(it.next().unwrap().data, 1);
        assert_eq!(it.next(), None);
        assert_eq!(item.data, 2);
        assert_eq!(tree.as_ref().front().unwrap().data, 1);
        assert_eq!(tree.as_mut().front_mut().unwrap().data, 1);
        assert_eq!(tree.as_ref().back().unwrap().data, 1);
        assert_eq!(tree.as_mut().back_mut().unwrap().data, 1);
        assert_eq!(tree.as_ref().len(), 1);
        assert_eq!(tree.as_ref().is_empty(), false);

        let item = X::from(tree.as_mut().pop_back()).unwrap();
        // []
        assert_eq!(item.data, 1);
        assert_eq!(tree.as_ref().front(), None);
        assert_eq!(tree.as_mut().front_mut(), None);
        assert_eq!(tree.as_ref().back(), None);
        assert_eq!(tree.as_mut().back_mut(), None);
        assert_eq!(tree.as_ref().len(), 0);
        assert_eq!(tree.as_ref().is_empty(), true);
    }

    #[test]
    fn test_insert_3() {
        let mut tree = Box::pin(BinTree::new(XLink));
        tree.as_mut().insert(X::new(2));
        tree.as_mut().insert(X::new(3));
        tree.as_mut().insert(X::new(1));

        // [2-3]
        // [  1]
        let mut it = tree.as_ref().iter();
        assert_eq!(it.next().unwrap().data, 1);
        assert_eq!(it.next().unwrap().data, 2);
        assert_eq!(it.next().unwrap().data, 3);
        assert_eq!(it.next(), None);
        assert_eq!(tree.as_ref().len(), 3);

        let item = X::from(tree.as_mut().pop_front()).unwrap();
        assert_eq!(item.data, 1);

        let item = X::from(tree.as_mut().pop_back()).unwrap();
        assert_eq!(item.data, 3);

        let item = X::from(tree.as_mut().pop_front()).unwrap();
        assert_eq!(item.data, 2);
    }
}
