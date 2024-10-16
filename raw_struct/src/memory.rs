use core::{
    mem::{
        self,
        MaybeUninit,
    },
    slice,
};

use crate::error::{
    self,
    AccessViolation,
};

pub trait MemoryView {
    fn read_memory(&self, offset: u64, buffer: &mut [u8]) -> Result<(), Box<dyn error::ErrorType>>;
}

impl<T: Copy> MemoryView for T {
    fn read_memory(
        &self,
        offset: u64,
        buffer: &mut [u8],
    ) -> Result<(), alloc::boxed::Box<dyn error::ErrorType>> {
        let src_buffer = unsafe {
            core::slice::from_raw_parts(self as *const _ as *const u8, core::mem::size_of_val(self))
        };

        let offset = offset as usize;
        if offset + buffer.len() > src_buffer.len() {
            return Err(Box::new(AccessViolation));
        }

        buffer.copy_from_slice(&src_buffer[offset..offset + buffer.len()]);
        Ok(())
    }
}

pub trait FromMemoryView: Sized {
    fn read_object(view: &dyn MemoryView, offset: u64) -> Result<Self, Box<dyn error::ErrorType>>;
    // fn read_boxed(view: &dyn MemoryView, offset: u64) -> Result<Box<Self>, Box<dyn error::ErrorType>>;
}

impl<T: Copy> FromMemoryView for T {
    fn read_object(view: &dyn MemoryView, offset: u64) -> Result<Self, Box<dyn error::ErrorType>> {
        let mut result = MaybeUninit::uninit();
        let size = mem::size_of_val(&result);

        let buffer = unsafe { slice::from_raw_parts_mut(&mut result as *mut _ as *mut u8, size) };
        view.read_memory(offset, buffer)?;

        Ok(unsafe { result.assume_init() })
    }
}
