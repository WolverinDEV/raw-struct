use core::{
    self,
    ops::Deref,
};

use crate::{
    memory::MemoryView,
    view::{
        Viewable,
        ViewableImplementation,
    },
    Copy,
    SizedViewable,
};

pub struct ReferenceMemory<M: MemoryView> {
    address: u64,
    memory: M,
}

impl<M: MemoryView> ReferenceMemory<M> {
    pub fn address(&self) -> u64 {
        self.address
    }

    pub fn memory_view(&self) -> &M {
        &self.memory
    }
}

impl<M: MemoryView> MemoryView for ReferenceMemory<M> {
    type AccessError = M::AccessError;

    fn read_memory(&self, offset: u64, buffer: &mut [u8]) -> Result<(), Self::AccessError> {
        self.memory.read_memory(self.address + offset, buffer)
    }
}

/// A reference to an object living in the underlying memory view.
pub struct Reference<T: Viewable, M: MemoryView> {
    inner: T::Implementation<ReferenceMemory<M>>,
}

impl<T: Viewable, M: MemoryView> Reference<T, M> {
    pub fn new(memory: M, address: u64) -> Self {
        Self {
            inner: T::from_memory(ReferenceMemory { address, memory }),
        }
    }

    pub fn reference_memory_address(&self) -> u64 {
        self.inner.memory_view().address()
    }

    pub fn reference_memory(&self) -> &M {
        self.inner.memory_view().memory_view()
    }

    pub fn cast<V: Viewable>(self) -> Reference<V, M> {
        Reference {
            inner: V::from_memory(self.inner.into_memory_view()),
        }
    }
}

impl<T: SizedViewable, M: MemoryView> Reference<T, M> {
    pub fn create_copy(&self) -> Result<Copy<T>, M::AccessError> {
        Copy::read_from_memory(self.memory_view(), 0x00)
    }
}

impl<T: Viewable, M: MemoryView> Deref for Reference<T, M> {
    type Target = T::Implementation<ReferenceMemory<M>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
