use core::{
    convert::Infallible,
    marker,
    mem::{
        self,
        MaybeUninit,
    },
    ops::{
        Deref,
        DerefMut,
    },
    slice,
};

use crate::{
    memory::FromMemoryView,
    MemoryView,
    OutOfBoundsViolation,
    Reference,
    ViewableSized,
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
#[derive(Clone)]
pub struct Copy<V: ViewableSized> {
    inner: Reference<V, CopyMemory<V::Memory>>,
}

impl<V: ViewableSized> Copy<V> {
    pub fn new(inner: V::Memory) -> Self {
        Self {
            inner: Reference::new(CopyMemory(inner), 0x00),
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
        Self::read_object(memory, offset).map_err(|err| err.into_access_error())
    }
}

impl<V: ViewableSized> Deref for Copy<V> {
    type Target = Reference<V, CopyMemory<V::Memory>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<V: ViewableSized> DerefMut for Copy<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<V: ViewableSized> FromMemoryView for Copy<V> {
    type DecodeError = Infallible;

    fn read_object<M: MemoryView>(
        view: &M,
        offset: u64,
    ) -> Result<Self, crate::MemoryDecodeError<M::AccessError, Self::DecodeError>> {
        Ok(Self::new(V::Memory::read_object(view, offset)?))
    }
}
