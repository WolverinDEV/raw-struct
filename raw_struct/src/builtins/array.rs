use alloc::{
    format,
    sync::Arc,
    vec::Vec,
};
use core::{
    self,
    marker,
    mem,
    ops::Range,
    slice,
};

use crate::{
    view::Copyable,
    AccessError,
    AccessMode,
    Copy,
    FromMemoryView,
    MemoryView,
    Reference,
};

#[allow(clippy::len_without_is_empty)]
pub trait Array<T: ?Sized> {
    fn start_address(&self) -> u64;

    fn len(&self) -> Option<usize>;
}

impl<T: FromMemoryView> dyn Array<T> {
    pub fn element_at<E>(
        &self,
        memory: &dyn MemoryView<Error = E>,
        index: usize,
    ) -> Result<T, AccessError<E>> {
        let offset = (index * mem::size_of::<T>()) as u64;
        T::read_object(memory, self.start_address() + offset).map_err(|err| AccessError {
            source: err,
            offset: self.start_address() + offset,
            size: mem::size_of::<T>(),
            mode: AccessMode::Read,
            object: "[..]".into(),
            member: Some(format!("[{}]", index).into()),
        })
    }

    pub fn elements<E>(
        &self,
        memory: &dyn MemoryView<Error = E>,
        range: Range<usize>,
    ) -> Result<Vec<T>, AccessError<E>> {
        let element_count = range.end - range.start;
        let mut result = Vec::with_capacity(element_count);

        unsafe {
            let buffer = slice::from_raw_parts_mut(
                result.as_mut_ptr() as *mut u8,
                element_count * mem::size_of::<T>(),
            );
            let offset = self.start_address() + (range.start * mem::size_of::<T>()) as u64;

            memory
                .read_memory(offset, buffer)
                .map_err(|err| AccessError {
                    source: err,
                    offset,
                    size: buffer.len(),
                    mode: AccessMode::Read,
                    object: "[..]".into(),
                    member: Some(format!("[{:#?}]", range).into()),
                })?;

            result.set_len(element_count);
        };

        Ok(result)
    }
}

/* We need T to be a Copyable as we need to know the element size */
impl<T: ?Sized + Copyable> dyn Array<T> {
    pub fn element_reference<E: 'static>(
        &self,
        memory: Arc<dyn MemoryView<Error = E>>,
        index: usize,
    ) -> Reference<T, E> {
        let offset = (index * T::MEMORY_SIZE) as u64;
        Reference::new(memory, self.start_address() + offset)
    }

    pub fn elements_reference<E: 'static>(
        &self,
        memory: Arc<dyn MemoryView<Error = E>>,
        range: Range<usize>,
    ) -> Vec<Reference<T, E>> {
        Vec::from_iter(range.map(|index| {
            Reference::new(
                memory.clone(),
                self.start_address() + (index * T::MEMORY_SIZE) as u64,
            )
        }))
    }
}

impl<T: ?Sized + Copyable> dyn Array<T> {
    pub fn element_copy<E>(
        &self,
        memory: &dyn MemoryView<Error = E>,
        index: usize,
    ) -> Result<Copy<T>, AccessError<E>>
    where
        Copy<T>: marker::Copy,
    {
        let offset = (index * T::MEMORY_SIZE) as u64;
        Copy::read_object(memory, self.start_address() + offset).map_err(|err| AccessError {
            source: err,
            offset: self.start_address() + offset,
            size: T::MEMORY_SIZE,
            mode: AccessMode::Read,
            object: format!("[{}]", T::name()).into(),
            member: Some(format!("[{}]", index).into()),
        })
    }

    pub fn elements_copy<E>(
        &self,
        memory: &dyn MemoryView<Error = E>,
        range: Range<usize>,
    ) -> Result<Vec<Copy<T>>, AccessError<E>> {
        let element_count = range.end - range.start;
        let mut result = Vec::<T::Memory>::with_capacity(element_count);

        unsafe {
            let buffer = slice::from_raw_parts_mut(
                result.as_mut_ptr() as *mut u8,
                element_count * T::MEMORY_SIZE,
            );
            let offset = self.start_address() + (range.start * T::MEMORY_SIZE) as u64;

            memory
                .read_memory(offset, buffer)
                .map_err(|err| AccessError {
                    source: err,
                    offset,
                    size: buffer.len(),
                    mode: AccessMode::Read,
                    object: "[..]".into(),
                    member: Some(format!("[{:#?}]", range).into()),
                })?;

            result.set_len(element_count);
        };

        Ok(result.into_iter().map(Copy::<T>::new).collect::<Vec<_>>())
    }
}

pub trait SizedArray<T: ?Sized, const N: usize>: Array<T> {}

impl<T: ?Sized, const N: usize> dyn SizedArray<T, N> {
    pub fn len(&self) -> usize {
        N
    }
}
