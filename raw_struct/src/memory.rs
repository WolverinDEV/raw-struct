use alloc::boxed::Box;
use core::{
    marker,
    mem::{
        self,
        MaybeUninit,
    },
    slice,
};

use crate::error::AccessViolation;

pub trait MemoryView: Send + Sync {
    type Error;

    fn read_memory(&self, offset: u64, buffer: &mut [u8]) -> Result<(), Self::Error>;
}

impl MemoryView for &[u8] {
    type Error = AccessViolation;

    fn read_memory(&self, offset: u64, buffer: &mut [u8]) -> Result<(), Self::Error> {
        let offset = offset as usize;
        if offset + buffer.len() > self.len() {
            return Err(AccessViolation);
        }

        buffer.copy_from_slice(&self[offset..offset + buffer.len()]);
        Ok(())
    }
}

#[derive(Clone)]
pub struct CopyMemoryView<T: marker::Copy> {
    inner: T,
}

impl<T: marker::Copy> CopyMemoryView<T> {
    pub fn new(value: T) -> Self {
        Self { inner: value }
    }
}

impl<T: marker::Copy + Send + Sync> MemoryView for CopyMemoryView<T> {
    type Error = AccessViolation;

    fn read_memory(&self, offset: u64, buffer: &mut [u8]) -> Result<(), Self::Error> {
        let src_buffer = unsafe {
            core::slice::from_raw_parts(
                &self.inner as *const _ as *const u8,
                core::mem::size_of_val(self),
            )
        };

        src_buffer.read_memory(offset, buffer)
    }
}

impl<T: marker::Copy + Send + Sync> From<T> for CopyMemoryView<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T: marker::Copy> marker::Copy for CopyMemoryView<T> {}

pub trait FromMemoryView: Sized {
    /// Read an instance of this byte by byte and interpret it as this.
    fn read_object<E>(view: &dyn MemoryView<Error = E>, offset: u64) -> Result<Self, E>;

    /// Read an instance of this byte by byte into heap memory
    fn read_boxed<E>(view: &dyn MemoryView<Error = E>, offset: u64) -> Result<Box<Self>, E>;
}

impl<T: Copy> FromMemoryView for T {
    fn read_object<E>(view: &dyn MemoryView<Error = E>, offset: u64) -> Result<Self, E> {
        let mut result = MaybeUninit::uninit();
        let size = mem::size_of_val(&result);

        // Safety:
        // As T is marker::Copy it is safe to access it's memory byte by byte
        let result = unsafe {
            let buffer = slice::from_raw_parts_mut(&mut result as *mut _ as *mut u8, size);
            view.read_memory(offset, buffer)?;
            result.assume_init()
        };
        Ok(result)
    }

    fn read_boxed<E>(view: &dyn MemoryView<Error = E>, offset: u64) -> Result<Box<Self>, E> {
        let mut result = Box::new_uninit();
        let size = mem::size_of_val(&result);

        // Safety:
        // As T is marker::Copy it is safe to access it's memory byte by byte
        let result = unsafe {
            let buffer = slice::from_raw_parts_mut(result.as_mut_ptr() as *mut _ as *mut u8, size);
            view.read_memory(offset, buffer)?;
            result.assume_init()
        };
        Ok(result)
    }
}
