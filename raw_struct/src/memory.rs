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

/// Decode an object from memory view
pub trait FromMemoryView: Sized {
    type DecodeError;

    fn read_object<M: MemoryView>(
        view: &M,
        offset: u64,
    ) -> Result<Self, MemoryDecodeError<M::AccessError, Self::DecodeError>>;

    // fn read_boxed(view: &dyn MemoryView, offset: u64) -> Result<Box<Self>, Box<dyn error::ErrorType>>;
}

macro_rules! impl_from_memory_copy {
    ($type:ty) => {
        impl FromMemoryView for $type {
            type DecodeError = Infallible;

            fn read_object<M: MemoryView>(
                view: &M,
                offset: u64,
            ) -> Result<Self, MemoryDecodeError<M::AccessError, Self::DecodeError>> {
                let mut result = MaybeUninit::uninit();

                {
                    let result_memory = unsafe {
                        slice::from_raw_parts_mut(
                            result.as_mut_ptr() as *mut u8,
                            mem::size_of::<$type>(),
                        )
                    };

                    view.read_memory(offset, result_memory)
                        .map_err(MemoryDecodeError::MemoryAccess)?;
                }

                Ok(unsafe { result.assume_init() })
            }
        }

        impl<const N: usize> FromMemoryView for [$type; N] {
            type DecodeError = Infallible;

            fn read_object<M: MemoryView>(
                view: &M,
                offset: u64,
            ) -> Result<Self, MemoryDecodeError<M::AccessError, Self::DecodeError>> {
                let mut result = MaybeUninit::uninit();

                {
                    let result_memory = unsafe {
                        slice::from_raw_parts_mut(
                            result.as_mut_ptr() as *mut u8,
                            mem::size_of::<$type>() * N,
                        )
                    };

                    view.read_memory(offset, result_memory)
                        .map_err(MemoryDecodeError::MemoryAccess)?;
                }

                Ok(unsafe { result.assume_init() })
            }
        }
    };
}

impl_from_memory_copy!(i8);
impl_from_memory_copy!(u8);
impl_from_memory_copy!(i16);
impl_from_memory_copy!(u16);
impl_from_memory_copy!(i32);
impl_from_memory_copy!(u32);
impl_from_memory_copy!(i64);
impl_from_memory_copy!(u64);

impl_from_memory_copy!(f32);
impl_from_memory_copy!(f64);

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
