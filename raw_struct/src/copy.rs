use core::{
    marker,
    mem::MaybeUninit,
    ops::Deref,
};

use crate::{
    memory::CopyMemoryView,
    view::Copyable,
    ViewableBase,
};

/// A Copy represents an owned copy of the struct binary contents
#[repr(transparent)]
pub struct Copy<T: ?Sized + Copyable> {
    inner: T::Instance<CopyMemoryView<T::Memory>>,
}

impl<T: ?Sized + Copyable> Copy<T> {
    pub fn new(inner: T::Memory) -> Self {
        Self {
            inner: T::create_view(CopyMemoryView::new(inner)),
        }
    }

    /// # Safety
    /// Creating a new instance of the target object based of zeros can lead to undefined
    /// behaviour as the internal state of the object may be invalid.
    pub unsafe fn new_zerod() -> Self {
        Self::new(MaybeUninit::zeroed().assume_init())
    }

    // pub fn read_object<E>(view: &dyn MemoryView<Error = E>, offset: u64) -> Result<Self, E> {
    //     Ok(Self {
    //         inner: T::create_view(CopyMemoryView::<T::Memory>::read_object(view, offset)?),
    //     })
    // }
}

impl<T: ?Sized + Copyable> Deref for Copy<T> {
    type Target = T::Instance<CopyMemoryView<T::Memory>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> Clone for Copy<T>
where
    T: ?Sized + Copyable,
{
    fn clone(&self) -> Self {
        Self {
            inner: T::create_view(self.inner.object_memory().clone()),
        }
    }
}

impl<T> marker::Copy for Copy<T>
where
    T: ?Sized + Copyable,
    T::Instance<CopyMemoryView<T::Memory>>: marker::Copy,
{
}
