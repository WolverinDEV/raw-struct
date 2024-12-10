use alloc::sync::Arc;
use core::{
    self,
    marker,
    ops::Deref,
};

use crate::{
    error::AccessError,
    memory::MemoryView,
    view::{
        Copyable,
        ViewableBase,
    },
    AccessMode,
    Copy,
    FromMemoryView,
    Viewable,
    ViewableInstance,
};

pub struct ReferenceMemory<MemoryError> {
    address: u64,
    inner: Arc<dyn MemoryView<Error = MemoryError>>,
}

impl<MemoryError> ReferenceMemory<MemoryError> {
    pub fn address(&self) -> u64 {
        self.address
    }

    pub fn memory_view(&self) -> &Arc<dyn MemoryView<Error = MemoryError>> {
        &self.inner
    }
}

impl<MemoryError> MemoryView for ReferenceMemory<MemoryError> {
    type Error = MemoryError;

    fn read_memory(&self, offset: u64, buffer: &mut [u8]) -> Result<(), MemoryError> {
        self.inner.read_memory(self.address + offset, buffer)
    }
}

impl<MemoryError> Clone for ReferenceMemory<MemoryError> {
    fn clone(&self) -> Self {
        Self {
            address: self.address,
            inner: self.inner.clone(),
        }
    }
}

/// A reference to an object living in the underlying memory view.
pub struct Reference<T: ?Sized + Viewable, MemoryError: 'static> {
    inner: T::Instance<ReferenceMemory<MemoryError>>,
}

impl<T: ?Sized + Viewable, MemoryError: 'static> Reference<T, MemoryError> {
    pub fn new(memory: Arc<dyn MemoryView<Error = MemoryError>>, address: u64) -> Self {
        Self {
            inner: T::create_view(ReferenceMemory {
                address,
                inner: memory,
            }),
        }
    }

    pub fn base_address(&self) -> u64 {
        let memory = self.inner.object_memory();
        memory.address
    }

    pub fn base_memory(&self) -> &Arc<dyn MemoryView<Error = MemoryError>> {
        let memory = self.inner.object_memory();
        &memory.inner
    }

    pub fn cast<V: ?Sized + Viewable>(&self) -> Reference<V, MemoryError> {
        Reference::new(self.base_memory().clone(), self.base_address())
    }
}

impl<T: ?Sized + Copyable, MemoryError: 'static> Reference<T, MemoryError>
where
    Copy<T>: marker::Copy,
{
    pub fn copy(&self) -> Result<Copy<T>, AccessError<MemoryError>> {
        let memory = self.base_memory().deref();
        Copy::<T>::read_object(memory, self.base_address()).map_err(|err| AccessError {
            source: err,

            object: T::name(),
            member: None,

            mode: AccessMode::Read,
            offset: self.base_address(),
            size: T::MEMORY_SIZE,
        })
    }
}

impl<T: Viewable + ?Sized, MemoryError> Deref for Reference<T, MemoryError> {
    type Target = T::Accessor<ReferenceMemory<MemoryError>>;

    fn deref(&self) -> &Self::Target {
        self.inner.get_accessor()
    }
}

impl<T: Viewable + ?Sized, MemoryError> Clone for Reference<T, MemoryError>
where
    T::Instance<ReferenceMemory<MemoryError>>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
