use alloc::vec::Vec;
use core::{
    self,
    mem,
    ops::Range,
    slice,
};

use crate::{
    Copy,
    CopyConstructable,
    FromMemoryView,
    MemoryDecodeError,
    MemoryView,
    Reference,
    SizedViewable,
};

#[allow(clippy::len_without_is_empty)]
pub trait Array<T> {
    fn start_address(&self) -> u64;

    fn len(&self) -> Option<usize>;
}

impl<T: FromMemoryView> dyn Array<T> {
    pub fn element_at<M: MemoryView>(
        &self,
        memory: &M,
        index: usize,
    ) -> Result<T, MemoryDecodeError<M::AccessError, T::DecodeError>> {
        let offset = (index * mem::size_of::<T>()) as u64;
        T::read_object(memory, self.start_address() + offset)
    }
}

impl<T: CopyConstructable> dyn Array<T> {
    pub fn elements<M: MemoryView>(
        &self,
        memory: &M,
        range: Range<usize>,
    ) -> Result<Vec<T>, M::AccessError> {
        let element_count = range.end - range.start;
        let mut result = Vec::with_capacity(element_count);

        let result_buffer = unsafe {
            slice::from_raw_parts_mut(
                result.as_mut_ptr() as *mut u8,
                element_count * mem::size_of::<T>(),
            )
        };
        memory.read_memory((range.start * mem::size_of::<T>()) as u64, result_buffer)?;
        unsafe {
            result.set_len(element_count);
        }

        Ok(result)
    }
}

impl<T: SizedViewable> dyn Array<T> {
    pub fn element_reference<M: MemoryView>(&self, memory: M, index: usize) -> Reference<T, M> {
        let offset = (index * T::memory_size()) as u64;
        Reference::new(memory, self.start_address() + offset)
    }

    pub fn elements_reference<M: MemoryView + Clone>(
        &self,
        memory: M,
        range: Range<usize>,
    ) -> Vec<Reference<T, M>> {
        Vec::from_iter(range.map(|index| {
            Reference::new(
                memory.clone(),
                self.start_address() + (index * T::memory_size()) as u64,
            )
        }))
    }

    pub fn element_copy<M: MemoryView>(
        &self,
        memory: &M,
        index: usize,
    ) -> Result<Copy<T>, M::AccessError> {
        let offset = (index * T::memory_size()) as u64;
        Copy::read_from_memory(memory, self.start_address() + offset)
    }

    pub fn elements_copy<M: MemoryView>(
        &self,
        memory: &M,
        range: Range<usize>,
    ) -> Result<Vec<Copy<T>>, M::AccessError> {
        let element_count = range.end - range.start;
        let mut result = Vec::<T::Memory>::with_capacity(element_count);

        unsafe {
            let buffer = slice::from_raw_parts_mut(
                result.as_mut_ptr() as *mut u8,
                element_count * T::memory_size(),
            );
            let offset = self.start_address() + (range.start * T::memory_size()) as u64;
            memory.read_memory(offset, buffer)?;
            result.set_len(element_count);
        };

        Ok(result.into_iter().map(Copy::<T>::new).collect::<Vec<_>>())
    }
}

pub trait SizedArray<T, const N: usize>: Array<T> {}

impl<T, const N: usize> dyn SizedArray<T, N> {
    pub fn len(&self) -> usize {
        N
    }
}
