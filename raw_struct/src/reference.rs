use alloc::sync::Arc;
use core::{
    self,
    marker,
    ops::Deref,
};

use crate::{
    error::AccessError,
    memory::MemoryView,
    view::ViewableBase,
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

    fn read_memory(&self, offset: u64, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.inner.read_memory(self.address + offset, buffer)
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

    // TODO: Rename base_address
    pub fn reference_address(&self) -> u64 {
        let memory = self.inner.object_memory();
        memory.address
    }

    // TODO: Rename base_memory
    pub fn reference_memory(&self) -> &Arc<dyn MemoryView<Error = MemoryError>> {
        let memory = self.inner.object_memory();
        &memory.inner
    }

    pub fn cast<V: ?Sized + Viewable>(&self) -> Reference<V, MemoryError> {
        Reference::new(self.reference_memory().clone(), self.reference_address())
    }
}

impl<T: ?Sized + Viewable, MemoryError: 'static> Reference<T, MemoryError>
where
    T::Instance<T::Memory>: marker::Copy,
{
    pub fn copy(&self) -> Result<Copy<T>, AccessError<MemoryError>> {
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

impl<T: Viewable + ?Sized, MemoryError> Deref for Reference<T, MemoryError> {
    type Target = T::Accessor<ReferenceMemory<MemoryError>>;

    fn deref(&self) -> &Self::Target {
        self.inner.get_accessor()
    }
}
