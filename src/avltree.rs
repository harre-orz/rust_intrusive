use super::Adapter;
use crate::ptr::{NonNullPtr, Pointer};
use std::marker::PhantomData;
use std::pin::Pin;
use std::ptr::NonNull;

// pub struct Link<T, P>
// where
//     T: Unpin,
//     P: Pointer<T>,
// {
//     height: usize,
//     left_ptr: Option<Pin<NonNullPtr<T, P>>>,
//     right_ptr: Option<Pin<NonNullPtr<T, P>>>,
// }

// pub struct AVLSet<T, A, P>
// where
//     T: Unpin,
//     P: Pointer<T>,
// {
//     link: Link<T, P>,
//     size: A,
// }

// pub struct AVLMultiSet<T, A, P>
// where
//     T: Unpin,
//     P: Pointer<T>,
// {
//     link: Link<T, P>,
//     size: A,
// }


// pub struct AVLMap<K, V, A, P>
// where
//     K: Unpin,
//     V: Unpin,
//     P: Pointer<(K, V)>,
// {
//     link: Link<(K, V), P>,
//     size: A,
// }

// pub struct AVLMultiMap<K, V, A, P>
// where
//     K: Unpin,
//     V: Unpin,
//     P: Pointer<(K, V)>,
// {
//     link: Link<(K, V), P>,
//     size: A,
// }

