use alloc::{
    format,
    sync::Arc,
    vec::Vec,
};
use core::{
    marker::{
        self,
    },
    mem,
    ops::Range,
    slice,
};

use crate::{
    view::MemoryViewEx,
    AccessError,
    AccessMode,
    Copy,
    MemoryView,
    Reference,
    Viewable,
};

pub trait Array<T: ?Sized> {
    fn start_address(&self) -> u64;
}

pub trait SizedArray<T: ?Sized, const N: usize>: Array<T> {}

impl<T: marker::Copy> dyn Array<T> {
    pub fn element_at(&self, memory: &dyn MemoryView, index: usize) -> Result<T, AccessError> {
        let offset = (index * mem::size_of::<T>()) as u64;
        T::from_memory(memory, self.start_address() + offset).map_err(|err| AccessError {
            source: err,
            offset: self.start_address() + offset,
            size: mem::size_of::<T>(),
            mode: AccessMode::Read,
            object: "[..]".into(),
            member: Some(format!("[{}]", index).into()),
        })
    }

    pub fn elements(
        &self,
        memory: &dyn MemoryView,
        range: Range<usize>,
    ) -> Result<Vec<T>, AccessError> {
        let element_count = range.end - range.start;
        let mut result = Vec::with_capacity(element_count);

        unsafe {
            let buffer = slice::from_raw_parts_mut(
                result.as_mut_ptr() as *mut u8,
                element_count * mem::size_of::<T>(),
            );
            let offset = self.start_address() + (range.start * mem::size_of::<T>()) as u64;

            memory.read(offset, buffer).map_err(|err| AccessError {
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

impl<T: ?Sized + Viewable<T>> dyn Array<T> {
    pub fn element_reference(&self, memory: Arc<dyn MemoryView>, index: usize) -> Reference<T> {
        let offset = (index * T::MEMORY_SIZE) as u64;
        Reference::new(memory, self.start_address() + offset)
    }

    pub fn elements_reference(
        &self,
        memory: Arc<dyn MemoryView>,
        range: Range<usize>,
    ) -> Vec<Reference<T>> {
        Vec::from_iter(range.map(|index| {
            Reference::new(
                memory.clone(),
                self.start_address() + (index * T::MEMORY_SIZE) as u64,
            )
        }))
    }
}

impl<T: ?Sized + Viewable<T>> dyn Array<T> {
    pub fn element_copy(
        &self,
        memory: &dyn MemoryView,
        index: usize,
    ) -> Result<Copy<T>, AccessError> {
        let offset = (index * T::MEMORY_SIZE) as u64;
        Ok(Copy::new(
            T::Memory::from_memory(memory, self.start_address() + offset).map_err(|err| {
                AccessError {
                    source: err,
                    offset: self.start_address() + offset,
                    size: T::MEMORY_SIZE,
                    mode: AccessMode::Read,
                    object: format!("[{}]", T::name()).into(),
                    member: Some(format!("[{}]", index).into()),
                }
            })?,
        ))
    }

    pub fn elements_copy(
        &self,
        memory: &dyn MemoryView,
        range: Range<usize>,
    ) -> Result<Vec<Copy<T>>, AccessError> {
        let element_count = range.end - range.start;
        let mut result = Vec::<T::Memory>::with_capacity(element_count);

        unsafe {
            let buffer = slice::from_raw_parts_mut(
                result.as_mut_ptr() as *mut u8,
                element_count * T::MEMORY_SIZE,
            );
            let offset = self.start_address() + (range.start * T::MEMORY_SIZE) as u64;

            memory.read(offset, buffer).map_err(|err| AccessError {
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
