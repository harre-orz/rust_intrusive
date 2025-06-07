use std::fmt;

pub trait Size: Default {
    fn incr(&mut self);

    fn decr(&mut self);

    fn len<I>(&self, it: I) -> usize
    where
        I: Iterator;

    fn is_empty<I>(&self, it: I) -> bool
    where
        I: Iterator;
}

pub trait LinkAdapter<T> {
    type Link;
    type Size: Size;

    fn link_ref(data: &T) -> &Self::Link;

    fn link_mut(data: &mut T) -> &mut Self::Link;
}

#[derive(Default)]
pub struct NumerateSize;

impl Size for NumerateSize {
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

impl fmt::Debug for NumerateSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#")
    }
}

#[derive(Default)]
pub struct LiterateSize(usize);

impl Size for LiterateSize {
    fn incr(&mut self) {
        self.0 += 1;
    }

    fn decr(&mut self) {
        self.0 -= 1;
    }

    fn len<I>(&self, _: I) -> usize
    where
        I: Iterator,
    {
        self.0
    }

    fn is_empty<I>(&self, _: I) -> bool
    where
        I: Iterator,
    {
        self.0 == 0
    }
}

impl fmt::Debug for LiterateSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
