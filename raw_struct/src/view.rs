use alloc::{
    borrow::Cow,
    boxed::Box,
};
use core::{
    self,
    error::Error,
    mem::{
        self,
        MaybeUninit,
    },
    slice,
};

use crate::error::AccessViolation;

pub trait MemoryView {
    fn read(&self, offset: u64, buffer: &mut [u8]) -> Result<(), Box<dyn Error>>;
}

impl<T: Copy> MemoryView for T {
    fn read(
        &self,
        offset: u64,
        buffer: &mut [u8],
    ) -> Result<(), alloc::boxed::Box<dyn core::error::Error>> {
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

pub trait MemoryViewEx: Sized {
    fn from_memory(view: &dyn MemoryView, offset: u64) -> Result<Self, Box<dyn Error>>;
    fn from_memory_boxed(view: &dyn MemoryView, offset: u64) -> Result<Box<Self>, Box<dyn Error>>;
    fn from_memory_boxed_in(
        view: &dyn MemoryView,
        offset: u64,
        target: &mut Box<Self>,
    ) -> Result<(), Box<dyn Error>>;
}

impl<T: Copy> MemoryViewEx for T {
    fn from_memory(view: &dyn MemoryView, offset: u64) -> Result<Self, Box<dyn Error>> {
        let mut result = MaybeUninit::uninit();
        let size = mem::size_of_val(&result);

        let buffer = unsafe { slice::from_raw_parts_mut(&mut result as *mut _ as *mut u8, size) };
        view.read(offset, buffer)?;

        Ok(unsafe { result.assume_init() })
    }

    fn from_memory_boxed(view: &dyn MemoryView, offset: u64) -> Result<Box<Self>, Box<dyn Error>> {
        let mut result = Box::new_uninit();
        MaybeUninit::<T>::from_memory_boxed_in(view, offset, &mut result)?;
        Ok(unsafe { result.assume_init() })
    }

    fn from_memory_boxed_in(
        view: &dyn MemoryView,
        offset: u64,
        target: &mut Box<Self>,
    ) -> Result<(), Box<dyn Error>> {
        let size = mem::align_of::<Self>();

        let buffer =
            unsafe { slice::from_raw_parts_mut(target.as_mut() as *mut _ as *mut u8, size) };
        view.read(offset, buffer)?;

        Ok(())
    }
}

pub trait ViewableBase {
    fn object_memory(&self) -> &dyn MemoryView;
}

pub trait ViewableImplementation<M: MemoryView, T: ?Sized> {
    fn object_memory(&self) -> &M;
    fn as_trait(&self) -> &T;
}

pub trait Viewable<T: ?Sized>: 'static {
    type Memory: Copy + MemoryView;
    type Implementation<M: MemoryView + 'static>: ViewableImplementation<M, T>;

    const MEMORY_SIZE: usize = core::mem::size_of::<Self::Memory>();

    fn create<M: MemoryView + 'static>(memory: M) -> Self::Implementation<M>;
    fn name() -> Cow<'static, str>;
}
