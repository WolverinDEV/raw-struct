use core::{
    self,
    convert::Infallible,
    mem::{
        self,
        MaybeUninit,
    },
    slice,
};

use crate::{
    error::OutOfBoundsViolation,
    MemoryDecodeError,
};

pub trait MemoryView {
    type AccessError;

    fn read_memory(&self, offset: u64, buffer: &mut [u8]) -> Result<(), Self::AccessError>;
}

impl<M: MemoryView> MemoryView for &M {
    type AccessError = M::AccessError;

    fn read_memory(&self, offset: u64, buffer: &mut [u8]) -> Result<(), Self::AccessError> {
        M::read_memory(self, offset, buffer)
    }
}

#[cfg(feature = "alloc")]
impl<M: ?Sized + MemoryView> MemoryView for alloc::sync::Arc<M> {
    type AccessError = M::AccessError;

    fn read_memory(&self, offset: u64, buffer: &mut [u8]) -> Result<(), Self::AccessError> {
        M::read_memory(self, offset, buffer)
    }
}

impl MemoryView for &[u8] {
    type AccessError = OutOfBoundsViolation;

    fn read_memory(&self, offset: u64, buffer: &mut [u8]) -> Result<(), Self::AccessError> {
        let offset = offset as usize;
        if offset + buffer.len() > self.len() {
            return Err(OutOfBoundsViolation {
                access_offset: offset,
                access_len: buffer.len(),

                src_len: self.len(),
            });
        }

        buffer.copy_from_slice(&self[offset..offset + buffer.len()]);
        Ok(())
    }
}

pub trait MemoryViewDereferenceable: MemoryView {
    fn dereference(&self, address: u64) -> Result<u64, Self::AccessError>;
}

impl<M: MemoryViewDereferenceable> MemoryViewDereferenceable for &M {
    fn dereference(&self, address: u64) -> Result<u64, Self::AccessError> {
        M::dereference(&self, address)
    }
}

#[cfg(feature = "alloc")]
impl<M: ?Sized + MemoryViewDereferenceable> MemoryViewDereferenceable for alloc::sync::Arc<M> {
    fn dereference(&self, address: u64) -> Result<u64, Self::AccessError> {
        M::dereference(&self, address)
    }
}

/// Decode an object from memory view.
///
/// Note:
/// The decoded object may be different in memory representation and size then the original.
pub trait FromMemoryView: Sized {
    type DecodeError;

    fn read_object<M: MemoryView>(
        view: &M,
        offset: u64,
    ) -> Result<Self, MemoryDecodeError<M::AccessError, Self::DecodeError>>;

    // fn read_boxed(view: &dyn MemoryView, offset: u64) -> Result<Box<Self>, Box<dyn error::ErrorType>>;
}

/// Marker trait for types that can be trivially constructed by copying their
/// underlying data. It can also be assumed, that the size of a CopyConstructable is the actual binary object size.
///
/// For types implementing this trait:
/// - [`FromMemoryView`] is automatically implemented.
/// - The associated [`DecodeError`] is fixed to [`Infallible`], since decoding
///   cannot fail.
pub trait CopyConstructable: Copy {}

impl<T: CopyConstructable> FromMemoryView for T {
    type DecodeError = Infallible;

    fn read_object<M: MemoryView>(
        view: &M,
        offset: u64,
    ) -> Result<Self, MemoryDecodeError<M::AccessError, Self::DecodeError>> {
        let mut result = MaybeUninit::uninit();

        let result_memory = unsafe {
            slice::from_raw_parts_mut(result.as_mut_ptr() as *mut u8, mem::size_of::<T>())
        };

        view.read_memory(offset, result_memory)
            .map_err(MemoryDecodeError::MemoryAccess)?;

        Ok(unsafe { result.assume_init() })
    }
}

impl<T1: CopyConstructable, T2: CopyConstructable> CopyConstructable for (T1, T2) {}
impl<T: CopyConstructable, const N: usize> CopyConstructable for [T; N] {}

impl CopyConstructable for u8 {}
impl CopyConstructable for i8 {}

impl CopyConstructable for u16 {}
impl CopyConstructable for i16 {}

impl CopyConstructable for u32 {}
impl CopyConstructable for i32 {}

impl CopyConstructable for u64 {}
impl CopyConstructable for i64 {}

impl CopyConstructable for f32 {}
impl CopyConstructable for f64 {}

impl FromMemoryView for bool {
    type DecodeError = Infallible;

    fn read_object<M: MemoryView>(
        view: &M,
        offset: u64,
    ) -> Result<Self, MemoryDecodeError<M::AccessError, Self::DecodeError>> {
        let value = u8::read_object(view, offset)?;
        Ok(value > 0)
    }
}

#[cfg(test)]
mod test {
    use crate::memory::FromMemoryView;

    #[test]
    fn test_typing() {
        let memory = &[0x01u8, 0x00, 0x00, 0x00];

        let x = u32::read_object(&memory.as_slice(), 0x00);
        assert_eq!(x, Ok(0x01));
    }
}
