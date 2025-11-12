use core::{
    self,
    marker::PhantomData,
};

use crate::{
    memory::MemoryView,
    Copy,
    FromMemoryView,
    MemoryDecodeError,
    Viewable,
    ViewableExtends,
    ViewableField,
    ViewableSized,
};

/// A reference to an object living in the underlying memory view.
pub struct Reference<V, M: MemoryView> {
    memory: M,
    memory_offset: u64,
    _type: PhantomData<V>,
}

impl<T, M: MemoryView> Reference<T, M> {
    pub fn new(memory: M, address: u64) -> Self {
        Self {
            memory,
            memory_offset: address,
            _type: Default::default(),
        }
    }

    pub fn reference_memory_address(&self) -> u64 {
        self.memory_offset
    }

    pub fn reference_memory(&self) -> &M {
        &self.memory
    }

    pub fn cast<V>(self) -> Reference<V, M> {
        Reference {
            memory: self.memory,
            memory_offset: self.memory_offset,
            _type: Default::default(),
        }
    }
}

impl<T: Viewable, M: MemoryView> Reference<T, M> {
    pub fn read_field<R: FromMemoryView, C>(
        &self,
        field: &ViewableField<C, R>,
    ) -> Result<R, MemoryDecodeError<M::AccessError, R::DecodeError>>
    where
        T: ViewableExtends<C>,
    {
        R::read_object(&self.memory, self.memory_offset + field.offset())
    }

    pub fn reference_field<R, C>(&self, field: &ViewableField<C, R>) -> Reference<R, &M>
    where
        T: ViewableExtends<C>,
    {
        Reference::new(&self.memory, self.memory_offset + field.offset())
    }
}

impl<T: FromMemoryView, M: MemoryView> Reference<T, M> {
    pub fn read(&self) -> Result<T, MemoryDecodeError<M::AccessError, T::DecodeError>> {
        T::read_object(&self.memory, self.memory_offset)
    }
}

impl<T: ViewableSized, M: MemoryView> Reference<T, M> {
    pub fn create_copy(&self) -> Result<Copy<T>, M::AccessError> {
        Copy::read_from_memory(&self.memory, self.memory_offset)
    }
}

impl<T, M: MemoryView + Clone> Clone for Reference<T, M> {
    fn clone(&self) -> Self {
        Self {
            memory: self.memory.clone(),
            memory_offset: self.memory_offset,
            _type: Default::default(),
        }
    }
}
