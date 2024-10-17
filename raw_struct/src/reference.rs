use alloc::sync::Arc;
use core::{
    self,
    marker,
    ops::Deref,
};

use crate::{
    error::{
        AccessError,
        Error,
    },
    memory::MemoryView,
    view::ViewableImplementation,
    AccessMode,
    Copy,
    FromMemoryView,
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
    fn read_memory(&self, offset: u64, buffer: &mut [u8]) -> Result<(), Error> {
        self.inner.read_memory(self.address + offset, buffer)
    }
}

/// A reference to an object living in the underlying memory view.
pub struct Reference<T: ?Sized + Viewable<T>> {
    inner: T::Implementation<ReferenceMemory>,
}

impl<T: ?Sized + Viewable<T>> Reference<T> {
    pub fn new(memory: Arc<dyn MemoryView>, address: u64) -> Self {
        Self {
            inner: T::create(ReferenceMemory {
                address,
                inner: memory,
            }),
        }
    }

    pub fn reference_address(&self) -> u64 {
        T::Implementation::memory(&self.inner).address()
    }

    pub fn reference_memory(&self) -> &Arc<dyn MemoryView> {
        T::Implementation::memory(&self.inner).memory_view()
    }

    pub fn cast<V: ?Sized + Viewable<V>>(&self) -> Reference<V> {
        Reference::<V>::new(self.reference_memory().clone(), self.reference_address())
    }
}

impl<T: ?Sized + Viewable<T>> Reference<T>
where
    T::Implementation<T::Memory>: marker::Copy,
{
    pub fn copy(&self) -> Result<Copy<T>, AccessError> {
        let memory = self.reference_memory().deref();
        Copy::read_object(memory, self.reference_address()).map_err(|err| AccessError {
            source: err,

            object: T::name(),
            member: None,

            mode: AccessMode::Read,
            offset: self.reference_address(),
            size: T::MEMORY_SIZE,
        })
    }
}

impl<T: Viewable<T> + ?Sized> Deref for Reference<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.as_trait()
    }
}
