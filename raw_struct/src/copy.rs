use core::{
    mem::{
        self,
        MaybeUninit,
    },
    ops::Deref,
};

use crate::{
    error::AccessMode,
    view::{
        MemoryViewEx,
        Viewable,
    },
    AccessError,
    MemoryView,
    ViewableImplementation,
};

/// A Copy represents an owned copy of the struct binary contents
#[derive(Clone)]
pub struct Copy<T: Viewable<T> + ?Sized> {
    inner: T::Implementation<T::Memory>,
}

impl<T: Viewable<T> + ?Sized> Copy<T> {
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

    pub fn from_memory(memory_view: &dyn MemoryView) -> Result<Self, AccessError> {
        let memory = T::Memory::from_memory(memory_view, 0x00).map_err(|err| AccessError {
            source: err,

            offset: 0x00,
            size: mem::size_of::<T::Memory>(),
            mode: AccessMode::Read,

            object: T::name(),
            member: None,
        })?;

        Ok(Copy::new(memory))
    }
}

impl<T: Viewable<T> + ?Sized> Deref for Copy<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.as_trait()
    }
}
