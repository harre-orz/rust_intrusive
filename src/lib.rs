use std::cmp::Ordering;

pub trait LinkAdapter<T>: Default {
    type Link;

    fn as_link_ref(data: &T) -> &Self::Link;

    fn as_link_mut(data: &mut T) -> &mut Self::Link;

    fn incr(&mut self) {}

    fn decr(&mut self) {}

    fn len<I>(&self, it: I) -> usize
    where
        I: Iterator,
    {
        it.count()
    }

    fn is_empty<I>(&self, mut it: I) -> bool
    where
        I: Iterator,
    {
        it.next().is_none()
    }
}

pub trait OrdAdapter<T>: LinkAdapter<T> {
    fn cmp(left: &T, right: &T) -> Ordering;
}

pub mod ptr;

pub mod slist;
pub mod list;
//pub mod bintree;
// pub mod avltree;
