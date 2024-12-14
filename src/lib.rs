
pub trait Sized : Default {
    fn increment(&mut self) {
    }
    
    fn decrement(&mut self) {
    }
    
    fn count<I>(&self, it: I) -> usize
    where
	I: Iterator
    {
	it.count()
    }
}

pub trait Adapter<T> : Sized
{
    type Link;
    
    fn as_link_ref(data: &T) -> &Self::Link;
    
    fn as_link_mut(data: &mut T) -> &mut Self::Link;
}


pub mod ptr;

pub mod slist;
