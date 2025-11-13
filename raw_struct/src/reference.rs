#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use core::{
    self,
    marker::PhantomData,
    mem,
};

use crate::{
    builtins::Ptr64,
    memory::{
        MemoryView,
        MemoryViewDereferenceable,
    },
    view::ViewableField,
    Copy,
    CopyConstructable,
    FromMemoryView,
    MemoryDecodeError,
    TypedViewableField,
    Viewable,
    ViewableExtends,
    ViewableSized,
};

/// A reference to an object living in the underlying memory view.
pub struct Reference<V: ?Sized, M: MemoryView> {
    memory: M,
    memory_offset: u64,
    _type: PhantomData<V>,
}

impl<T: ?Sized, M: MemoryView> Reference<T, M> {
    pub fn new(memory: M, address: u64) -> Self {
        Self {
            memory,
            memory_offset: address,
            _type: Default::default(),
        }
    }

    pub fn memory_address(&self) -> u64 {
        self.memory_offset
    }

    pub fn memory(&self) -> &M {
        &self.memory
    }

    pub fn cast<V: ?Sized>(self) -> Reference<V, M> {
        Reference {
            memory: self.memory,
            memory_offset: self.memory_offset,
            _type: Default::default(),
        }
    }
}

impl<T: ?Sized, M: MemoryView + Clone> Clone for Reference<T, M> {
    fn clone(&self) -> Self {
        Self {
            memory: self.memory.clone(),
            memory_offset: self.memory_offset,
            _type: Default::default(),
        }
    }
}

impl<T: Viewable, M: MemoryView> Reference<T, M> {
    pub fn read_field<R: FromMemoryView, C>(
        &self,
        field: &TypedViewableField<C, R>,
    ) -> Result<R, MemoryDecodeError<M::AccessError, R::DecodeError>>
    where
        T: ViewableExtends<C>,
    {
        R::read_object(&self.memory, self.memory_offset + field.offset())
    }

    pub fn reference_field<R, C>(&self, field: &TypedViewableField<C, R>) -> Reference<R, &M>
    where
        T: ViewableExtends<C>,
    {
        Reference::new(&self.memory, self.memory_offset + field.offset())
    }
}

impl<T: Viewable, M: MemoryView> Reference<T, M>
where
    for<'a> &'a M: MemoryViewDereferenceable,
{
    pub fn dereference_field<R: ?Sized, C>(
        &self,
        field: &TypedViewableField<C, Ptr64<R>>,
    ) -> Result<Reference<R, &M>, <&M as MemoryView>::AccessError>
    where
        T: ViewableExtends<C>,
    {
        self.reference_field(field).dereference()
    }
}

impl<T: ?Sized, M: MemoryViewDereferenceable> Reference<Ptr64<T>, M> {
    pub fn dereference(self) -> Result<Reference<T, M>, M::AccessError> {
        let ptr_value = self.read().map_err(|err| err.into_access_error())?;
        let memory_offset = self.memory.dereference(ptr_value.address())?;
        Ok(Reference::new(self.memory, memory_offset))
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

impl<T: CopyConstructable, M: MemoryView> Reference<[T], M> {
    pub fn read_element(&self, index: usize) -> Result<T, M::AccessError> {
        let element_offset = (index * mem::size_of::<T>()) as u64;
        T::read_object(&self.memory, self.memory_offset + element_offset)
            .map_err(|err| err.into_access_error())
    }
}

impl<T: ViewableSized, M: MemoryView> Reference<[T], M> {
    pub fn reference_element(&self, index: usize) -> Reference<T, &M> {
        let element_offset = (index * T::memory_size()) as u64;
        Reference::new(&self.memory, self.memory_offset + element_offset)
    }

    pub fn copy_element(&self, index: usize) -> Result<Copy<T>, M::AccessError> {
        let element_offset = (index * T::memory_size()) as u64;
        Copy::<T>::read_from_memory(&self.memory, self.memory_offset + element_offset)
    }
}

impl<T, M: MemoryView, const N: usize> Reference<[T; N], M> {
    pub fn len(&self) -> usize {
        N
    }
}

impl<T: CopyConstructable, M: MemoryView, const N: usize> Reference<[T; N], M> {
    pub fn read_element(&self, index: usize) -> Result<T, M::AccessError> {
        assert!(index <= N);

        let element_offset = (index * mem::size_of::<T>()) as u64;
        T::read_object(&self.memory, self.memory_offset + element_offset)
            .map_err(|err| err.into_access_error())
    }

    #[cfg(feature = "alloc")]
    pub fn read_elements(&self) -> Result<Vec<T>, M::AccessError> {
        let mut buffer = Vec::new();
        unsafe {
            buffer.reserve_exact(N);

            self.memory.read_memory(
                self.memory_offset,
                core::slice::from_raw_parts_mut(
                    buffer.as_mut_ptr() as *mut u8,
                    N * mem::size_of::<T>(),
                ),
            )?;

            buffer.set_len(N);
        }

        Ok(buffer)
    }
}

impl<T: ViewableSized, M: MemoryView, const N: usize> Reference<[T; N], M> {
    pub fn reference_element(&self, index: usize) -> Reference<T, &M> {
        let element_offset = (index * T::memory_size()) as u64;
        Reference::new(&self.memory, self.memory_offset + element_offset)
    }

    #[cfg(feature = "alloc")]
    pub fn reference_elements(&self) -> Vec<Reference<T, &M>> {
        let mut buffer = Vec::new();
        buffer.reserve_exact(N);

        for index in 0..self.len() {
            buffer.push(self.reference_element(index));
        }

        buffer
    }

    pub fn copy_element(&self, index: usize) -> Result<Copy<T>, M::AccessError> {
        let element_offset = (index * T::memory_size()) as u64;
        Copy::<T>::read_from_memory(&self.memory, self.memory_offset + element_offset)
    }

    #[cfg(feature = "alloc")]
    pub fn copy_elements(&self) -> Result<Vec<Copy<T>>, M::AccessError> {
        let mut buffer = Vec::new();
        buffer.reserve_exact(N);

        for index in 0..self.len() {
            buffer.push(self.copy_element(index)?);
        }

        Ok(buffer)
    }
}
