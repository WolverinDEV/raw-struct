use core::{
    convert::Infallible,
    marker,
    mem::{
        self,
        MaybeUninit,
    },
    ops::Deref,
    slice,
};

use crate::{
    FromMemoryView,
    MemoryDecodeError,
    MemoryView,
    OutOfBoundsViolation,
    Reference,
    SizedViewable,
    ViewableImplementation,
};

#[derive(Clone, Copy)]
pub struct CopyMemory<M: marker::Copy>(pub M);

impl<M: marker::Copy> MemoryView for CopyMemory<M> {
    type AccessError = OutOfBoundsViolation;

    fn read_memory(&self, offset: u64, buffer: &mut [u8]) -> Result<(), Self::AccessError> {
        let memory =
            unsafe { slice::from_raw_parts(&self.0 as *const _ as *const u8, mem::size_of::<M>()) };

        memory.read_memory(offset, buffer)
    }
}

/// A Copy represents an owned copy of the struct binary contents
#[repr(transparent)]
pub struct Copy<T: SizedViewable> {
    inner: T::Implementation<CopyMemory<T::Memory>>,
}

impl<T: SizedViewable> Copy<T> {
    pub fn new(inner: T::Memory) -> Self {
        Self {
            inner: T::from_memory(CopyMemory(inner)),
        }
    }

    /// # Safety
    /// Creating a new instance of the target object based of zeros can lead to undefined
    /// behaviour as the internal state of the object may be invalid.
    pub unsafe fn new_zerod() -> Self {
        Self::new(MaybeUninit::zeroed().assume_init())
    }

    pub fn read_from_memory<M: MemoryView>(
        memory: &M,
        offset: u64,
    ) -> Result<Self, M::AccessError> {
        let mut copy_memory = unsafe { MaybeUninit::<T::Memory>::uninit().assume_init() };
        memory.read_memory(offset, unsafe {
            slice::from_raw_parts_mut(
                &mut copy_memory as *mut _ as *mut u8,
                mem::size_of::<T::Memory>(),
            )
        })?;

        Ok(Self::new(copy_memory))
    }

    pub fn as_reference(&self) -> Reference<T, &CopyMemory<T::Memory>> {
        Reference::new(self.inner.memory_view(), 0x00)
    }
}

impl<T: SizedViewable> FromMemoryView for Copy<T> {
    type DecodeError = Infallible;

    fn read_object<M: MemoryView>(
        view: &M,
        offset: u64,
    ) -> Result<Self, MemoryDecodeError<M::AccessError, Self::DecodeError>> {
        Self::read_from_memory(view, offset).map_err(MemoryDecodeError::MemoryAccess)
    }
}

impl<T> Clone for Copy<T>
where
    T: SizedViewable,
    T::Implementation<CopyMemory<T::Memory>>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> marker::Copy for Copy<T>
where
    T: SizedViewable,
    T::Implementation<CopyMemory<T::Memory>>: marker::Copy,
{
}

impl<T: SizedViewable> Deref for Copy<T> {
    type Target = T::Implementation<CopyMemory<T::Memory>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
