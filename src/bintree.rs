use crate::{Adapter, OrdAdapter};
use crate::ptr::{NonNullPtr, Pointer};
use std::marker::PhantomData;
use std::pin::Pin;
use std::ptr::NonNull;
use std::cmp::Ordering;
use std::fmt;

pub struct Link<T, P = NonNull<T>>
{
    top_ptr: Option<Pin<NonNullPtr<T, P>>>,
    left_ptr: Option<Pin<NonNullPtr<T, P>>>,
    right_ptr: Option<Pin<NonNullPtr<T, P>>>,
}

impl<T, P> Link<T, P>
{
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

    fn has_left(&self) -> bool {
	self.left_ptr.is_some()
    }
}

impl<T, P> Default for Link<T, P>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, P> Unpin for Link<T, P>
where
    T: Unpin,
{
}

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
    item_ptr: *const Option<Pin<NonNullPtr<T, P>>>,
    _marker: PhantomData<&'a A>,
}

impl<'a, T, A, P> Iterator for Iter<'a, T, A, P>
where
    T: Unpin + 'a,
    P: Pointer<T> + 'a,
    A: Adapter<T, Link = Link<T, P>>,
{
    type Item = Pin<&'a T>;

    fn next(&mut self) -> Option<Self::Item> {
	if let Some(item) = unsafe { &*self.item_ptr } {
	    let item: &T = item;

	    self.item_ptr = &A::as_link_ref(item).right_ptr;
	    if let Some(right) = unsafe { &*self.item_ptr } {
		// right
		let mut node: *const _ = A::as_link_ref(right);
		while let Some(item) = &unsafe { &*node }.left_ptr {
		    self.item_ptr = &unsafe { &*node }.left_ptr;
		    node = A::as_link_ref(item);
		}
	    } else {
		// top
		self.item_ptr = &A::as_link_ref(item).top_ptr;
		while let Some(top) = unsafe { &*self.item_ptr } {
		    if A::as_link_ref(top).is_left(item) {
			break
		    } else {
			self.item_ptr = &A::as_link_ref(top).top_ptr;
		    }
		}
	    }
	    Some(Pin::new(item))
	} else {
	    None
	}
    }
}

impl<'a, T, A, P> DoubleEndedIterator for Iter<'a, T, A, P>
where
    T: Unpin + 'a,
    P: Pointer<T> + 'a,
    A: Adapter<T, Link = Link<T, P>>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
	if let Some(item) = unsafe { &*self.item_ptr } {
	    let item: &T = item;

	    self.item_ptr = &A::as_link_ref(item).left_ptr;
	    if let Some(left) = unsafe { &*self.item_ptr } {
		// left
		let mut node: *const _ = A::as_link_ref(left);
		while let Some(item) = &unsafe { &*node }.right_ptr {
		    self.item_ptr = &unsafe { &*node }.right_ptr;
		    node = A::as_link_ref(item);
		}
	    } else {
		// top
		self.item_ptr = &A::as_link_ref(item).top_ptr;
		while let Some(top) = unsafe { &*self.item_ptr } {
		    if A::as_link_ref(top).is_right(item) {
			break
		    } else {
			self.item_ptr = &A::as_link_ref(top).top_ptr;
		    }
		}
	    }
	    Some(Pin::new(item))
	} else {
	    None
	}
    }
}


pub struct IterMut<'a, T, A, P> {
    item_ptr: *mut Option<Pin<NonNullPtr<T, P>>>,
    _marker: PhantomData<&'a A>,
}

impl<'a, T, A, P> Iterator for IterMut<'a, T, A, P>
where
    T: Unpin + 'a,
    P: Pointer<T> + 'a,
    A: Adapter<T, Link = Link<T, P>>,
{
    type Item = Pin<&'a mut T>;

    fn next(&mut self) -> Option<Self::Item> {
	if let Some(item) = unsafe { &mut *self.item_ptr } {
	    let item: &mut T = item;

	    self.item_ptr = &mut A::as_link_mut(item).right_ptr;
	    if let Some(right) = unsafe { &mut *self.item_ptr } {
		// right
		let mut node: *mut _ = A::as_link_mut(right);
		while let Some(item) = &mut unsafe { &mut *node }.left_ptr {
		    self.item_ptr = &mut unsafe { &mut *node }.left_ptr;
		    node = A::as_link_mut(item);
		}
	    } else {
		// top
		self.item_ptr = &mut A::as_link_mut(item).top_ptr;
		while let Some(top) = unsafe { &mut *self.item_ptr } {
		    if A::as_link_ref(top).is_left(item) {
			break
		    } else {
			self.item_ptr = &mut A::as_link_mut(top).top_ptr;
		    }
		}
	    }
	    Some(Pin::new(item))
	} else {
	    None
	}
    }
}

impl<'a, T, A, P> DoubleEndedIterator for IterMut<'a, T, A, P>
where
    T: Unpin + 'a,
    P: Pointer<T> + 'a,
    A: Adapter<T, Link = Link<T, P>>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
	if let Some(item) = unsafe { &mut *self.item_ptr } {
	    let item: &mut T = item;

	    self.item_ptr = &mut A::as_link_mut(item).left_ptr;
	    if let Some(left) = unsafe { &mut *self.item_ptr } {
		// left
		let mut node: *mut _ = A::as_link_mut(left);
		while let Some(item) = &mut unsafe { &mut *node }.right_ptr {
		    self.item_ptr = &mut unsafe { &mut *node }.right_ptr;
		    node = A::as_link_mut(item);
		}
	    } else {
		// top
		self.item_ptr = &mut A::as_link_mut(item).top_ptr;
		while let Some(top) = unsafe { &mut *self.item_ptr } {
		    if A::as_link_mut(top).is_right(item) {
			break
		    } else {
			self.item_ptr = &mut A::as_link_mut(top).top_ptr;
		    }
		}
	    }
	    Some(Pin::new(item))
	} else {
	    None
	}
    }
}

pub struct BinTree<T, A, P = NonNull<T>>
where
{
    root: Option<Pin<NonNullPtr<T, P>>>,
    size: A,
}

impl<T, A, P> BinTree<T, A, P>
{
    pub const fn new(adapter: A) -> Self {
        Self {
            root: None,
            size: adapter,
        }
    }
}

impl<T, A, P> BinTree<T, A, P>
where
    T: Unpin,
    P: Pointer<T>,
    A: Adapter<T, Link = Link<T, P>> + OrdAdapter<T>,
{
    pub fn get(self: Pin<&Self>, data: &T) -> Option<Pin<&T>>
    {
	let self_ = Pin::into_inner(self);
	let mut node_ptr: *const Option<Pin<NonNullPtr<T, P>>> = &self_.root;
	loop {
	    if let Some(node) = unsafe { &*node_ptr } {
		match A::cmp(node.as_ref().get_ref(), data) {
		    Ordering::Equal =>
			return Some(Pin::new(node)),
		    Ordering::Less => {
			let link: &Link<T, P> = A::as_link_ref(node);
			node_ptr = &link.left_ptr
		    },
		    Ordering::Greater => {
			let link: &Link<T, P> = A::as_link_ref(node);
			node_ptr = &link.right_ptr
		    },
		}
	    } else {
		return None
	    }
	}
    }

    pub fn get_mut(self: Pin<&mut Self>, data: &T) -> Option<Pin<&mut T>>
    {
	let self_ = Pin::into_inner(self);
	let mut node_ptr: *mut Option<Pin<NonNullPtr<T, P>>> = &mut self_.root;
	loop {
	    if let Some(node) = unsafe { &mut *node_ptr } {
		match A::cmp(node.as_ref().get_ref(), data) {
		    Ordering::Equal =>
			return Some(Pin::new(node)),
		    Ordering::Less => {
			let link: &mut Link<T, P> = A::as_link_mut(node);
			node_ptr = &mut link.left_ptr
		    },
		    Ordering::Greater =>  {
			let link: &mut Link<T, P> = A::as_link_mut(node);
			node_ptr = &mut link.right_ptr
		    },
		}
	    } else {
		return None
	    }
	}
    }

    pub fn insert(self: Pin<&mut Self>, mut data: NonNull<T>) -> Option<NonNull<T>>
    {
	let data_link = A::as_link_mut(unsafe { data.as_mut() });
        debug_assert_eq!(data_link.is_linked(), false);

	let self_ = Pin::into_inner(self);
	self_.size.increment();
	let mut item: *mut T = if let Some(item) = &mut self_.root {
	    item.as_mut().get_mut()
	} else {
	    NonNullPtr::set(&mut self_.root, data);
	    return None
	};
	loop {
	    let item_ptr = match unsafe { A::cmp(&*item, data.as_ref()) } {
		Ordering::Equal => {
		    unsafe {
			let link = A::as_link_mut(&mut *item);
			if let Some(top) = &mut link.top_ptr {
			    let top: &mut T = top;
			    let top_link = A::as_link_mut(top);
			    if top_link.is_left(&mut *item) {
				// left
				NonNullPtr::set(&mut top_link.left_ptr, data);
			    } else {
				// right
				NonNullPtr::set(&mut top_link.right_ptr, data);
			    }
			    NonNullPtr::set(&mut data_link.top_ptr, NonNull::new_unchecked(top));
			};
			if let Some(left) = &mut link.left_ptr {
			    let left: &mut T = left;
			    NonNullPtr::set(&mut A::as_link_mut(left).top_ptr, data);
			    NonNullPtr::set(&mut data_link.left_ptr, NonNull::new_unchecked(left));
			};
			if let Some(right) = &mut link.right_ptr {
			    let right: &mut T = right;
			    NonNullPtr::set(&mut A::as_link_mut(right).top_ptr, data);
			    NonNullPtr::set(&mut data_link.right_ptr, NonNull::new_unchecked(right));
			};
			link.unlink();
			return Some(NonNull::new_unchecked(item));
		    }
		},
		Ordering::Greater => {
		    let item = unsafe { &mut *item };
		    let link: &mut Link<T, P> = A::as_link_mut(item);
		    &mut link.left_ptr
		}
		Ordering::Less => {
		    let item = unsafe { &mut *item };
		    let link: &mut Link<T, P> = A::as_link_mut(item);
		    &mut link.right_ptr
		}
	    };
	    if let Some(item_) = item_ptr {
		item = item_.as_mut().get_mut();
	    } else {
		let item = unsafe { NonNull::new_unchecked(item) };
		NonNullPtr::set(&mut data_link.top_ptr, item);
		NonNullPtr::set(item_ptr, data);
		return None
	    }
	}
    }
    
    pub fn remove(self: Pin<&mut Self>, _data: &T) -> Option<NonNull<T>> {
	None
    }


    pub fn iter(self: Pin<&Self>) -> Iter<T, A, P> {
	let self_ = Pin::into_inner(self);
	let mut item_ptr: *const _ = &self_.root;
	while let Some(item) = unsafe { &*item_ptr } {
	    if let Some(_) = &A::as_link_ref(item).left_ptr {
		item_ptr = &A::as_link_ref(item).left_ptr;
	    } else {
		break
	    }
	}
	Iter {
	    item_ptr: item_ptr,
	    _marker: PhantomData,
	}
    }

    pub fn is_empty(self: Pin<&Self>) -> bool {
	self.root.is_none()
    }

    pub fn len(self: Pin<&Self>) -> usize {
	self.size.count(self.iter())
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

impl<T, A, P> Unpin for BinTree<T, A, P>
where
    T: Unpin,
{
}

impl<T, A, P> fmt::Debug for BinTree<T, A, P>
where
    T: Unpin + fmt::Debug,
    P: Pointer<T>,
    A: fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
	if let Some(root) = &self.root {
	    write!(fmt, "BinTree {{ size: {:?}, root: ({:p}) {:?} }}", self.size, root.as_ref(), root.as_ref())
	} else {
	    write!(fmt, "BinTree {{ size: {:?}, root: None }}", self.size)
	}
    }
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

    #[derive(Debug, Default)]
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

    impl OrdAdapter<X> for XLink
    {
	fn cmp(left: &X, right: &X) -> Ordering {
	    left.x.cmp(&right.x)
	}
    }
    
    #[test]
    fn test() {
	let mut tree = Box::pin(BinTree::new(XLink));
	tree.as_mut().insert(X::new(1));
	
	{
	    println!("{:?}", tree);
	    let mut it = tree.as_ref().iter();
	    assert_eq!(it.next().unwrap().x, 1);
	}
	tree.as_mut().insert(X::new(2));
	tree.as_mut().insert(X::new(0));
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
