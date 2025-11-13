use core::marker::{
    self,
    PhantomData,
};

use crate::{
    Copy,
    CopyConstructable,
    FromMemoryView,
    MemoryDecodeError,
    MemoryView,
    Reference,
    Viewable,
    ViewableSized,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ptr64<T: ?Sized> {
    address: u64,
    _type: PhantomData<T>,
}

impl<T: ?Sized> Clone for Ptr64<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> marker::Copy for Ptr64<T> {}

impl<T: ?Sized> CopyConstructable for Ptr64<T> {}

impl<T: ?Sized> Ptr64<T> {
    pub const fn address(&self) -> u64 {
        self.address
    }

    pub const fn is_null(&self) -> bool {
        self.address == 0
    }

    pub const fn cast<V>(&self) -> Ptr64<V> {
        Ptr64::<V> {
            address: self.address,
            _type: PhantomData {},
        }
    }
}

impl<T: FromMemoryView> Ptr64<T> {
    /// Create a copy of the value the pointer points to
    #[must_use = "copied result must be used"]
    pub fn read_value<M: MemoryView>(
        &self,
        memory: &M,
    ) -> Result<Option<T>, MemoryDecodeError<M::AccessError, T::DecodeError>> {
        (self.address > 0)
            .then(|| T::read_object(memory, self.address))
            .transpose()
    }
}

impl<T: Viewable> Ptr64<T> {
    #[must_use]
    pub fn reference_value<M: MemoryView>(&self, memory: M) -> Option<Reference<T, M>> {
        (self.address > 0).then(|| Reference::new(memory, self.address))
    }
}

impl<T: ViewableSized> Ptr64<T> {
    /// Create a copy of the value the pointer points to
    #[must_use = "copied result must be used"]
    pub fn copy_value<M: MemoryView>(&self, memory: &M) -> Result<Option<Copy<T>>, M::AccessError> {
        (self.address > 0)
            .then(|| Copy::<T>::read_from_memory(memory, self.address))
            .transpose()
    }
}
