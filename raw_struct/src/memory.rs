use core::{
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

/*
 * Attention:
 * This is dangarous MemoryView should only explicitly be implemented because
 * this implementation allows to read from &0 (aka references) as 8 byte values.
 */
impl<T: Copy + Send + Sync> MemoryView for T {
    type Error = AccessViolation;

    fn read_memory(&self, offset: u64, buffer: &mut [u8]) -> Result<(), Self::Error> {
        let src_buffer = unsafe {
            core::slice::from_raw_parts(self as *const _ as *const u8, core::mem::size_of_val(self))
        };

        let offset = offset as usize;
        if offset + buffer.len() > src_buffer.len() {
            return Err(AccessViolation);
        }

        buffer.copy_from_slice(&src_buffer[offset..offset + buffer.len()]);
        Ok(())
    }
}

pub trait FromMemoryView: Sized {
    fn read_object<E>(view: &dyn MemoryView<Error = E>, offset: u64) -> Result<Self, E>;
    // fn read_boxed(view: &dyn MemoryView, offset: u64) -> Result<Box<Self>, Box<dyn error::ErrorType>>;
}

impl<T: Copy> FromMemoryView for T {
    fn read_object<E>(view: &dyn MemoryView<Error = E>, offset: u64) -> Result<Self, E> {
        let mut result = MaybeUninit::uninit();
        let size = mem::size_of_val(&result);

        let buffer = unsafe { slice::from_raw_parts_mut(&mut result as *mut _ as *mut u8, size) };
        view.read_memory(offset, buffer)?;

        Ok(unsafe { result.assume_init() })
    }
}
