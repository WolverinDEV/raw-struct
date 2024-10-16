use core::{
    marker,
    mem::MaybeUninit,
    ops::Deref,
};

use crate::{
    view::Viewable,
    ViewableImplementation,
};

/// A Copy represents an owned copy of the struct binary contents
#[repr(transparent)]
pub struct Copy<T: ?Sized + Viewable<T>> {
    inner: T::Implementation<T::Memory>,
}

impl<T: ?Sized + Viewable<T>> Copy<T> {
    pub fn new(inner: T::Memory) -> Self {
        Self {
            inner: T::create(inner),
        }
    }

    /// # Safety
    /// Creating a new instance of the target object based of zeros can lead to undefined
    /// behaviour as the internal state of the object may be invalid.
    pub unsafe fn new_zerod() -> Self {
        Self::new(MaybeUninit::zeroed().assume_init())
    }
}

impl<T: ?Sized + Viewable<T>> Deref for Copy<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.as_trait()
    }
}

impl<T> Clone for Copy<T>
where
    T: ?Sized + Viewable<T>,
    T::Implementation<T::Memory>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> marker::Copy for Copy<T>
where
    T: ?Sized + Viewable<T>,
    T::Implementation<T::Memory>: marker::Copy,
{
}
