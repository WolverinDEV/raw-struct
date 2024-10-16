use alloc::{
    boxed::Box,
    sync::Arc,
};
use core::{
    self,
    error::Error,
    ops::Deref,
};

use crate::{
    error::AccessError,
    view::{
        MemoryView,
        ViewableImplementation,
    },
    Copy,
    Viewable,
};

pub struct ReferenceMemory {
    address: u64,
    inner: Arc<dyn MemoryView>,
}

impl ReferenceMemory {
    pub fn address(&self) -> u64 {
        self.address
    }

    pub fn memory_view(&self) -> &Arc<dyn MemoryView> {
        &self.inner
    }
}

impl MemoryView for ReferenceMemory {
    fn read(&self, offset: u64, buffer: &mut [u8]) -> Result<(), Box<dyn Error>> {
        self.inner.read(self.address + offset, buffer)
    }
}

/// A reference to an object living in the underlying memory view.
pub struct Reference<T: Viewable<T> + ?Sized> {
    inner: T::Implementation<ReferenceMemory>,
}

impl<T: Viewable<T> + ?Sized> Reference<T> {
    pub fn new(address: u64, memory: Arc<dyn MemoryView>) -> Self {
        Self {
            inner: T::create(ReferenceMemory {
                address,
                inner: memory,
            }),
        }
    }

    pub fn reference_address(&self) -> u64 {
        self.inner.object_memory().address()
    }

    pub fn reference_memory(&self) -> &Arc<dyn MemoryView> {
        self.inner.object_memory().memory_view()
    }

    pub fn copy(&self) -> Result<Copy<T>, AccessError> {
        Copy::from_memory(self.inner.object_memory())
    }
}

impl<T: Viewable<T> + ?Sized> Deref for Reference<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.as_trait()
    }
}
