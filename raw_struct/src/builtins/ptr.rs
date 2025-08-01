use core::{
    marker::{
        self,
        PhantomData,
    },
    ops::Deref,
};

use crate::{
    builtins::Array,
    Copy,
    FromMemoryView,
    MemoryDecodeError,
    MemoryView,
    Reference,
    SizedViewable,
    Viewable,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ptr64<T: ?Sized> {
    pub address: u64,
    _type: PhantomData<T>,
}

impl<T: ?Sized> Clone for Ptr64<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: ?Sized> marker::Copy for Ptr64<T> {}

impl<T: ?Sized> Ptr64<T> {
    pub fn is_null(&self) -> bool {
        self.address == 0
    }

    pub fn cast<V>(&self) -> Ptr64<V> {
        Ptr64::<V> {
            address: self.address,
            _type: Default::default(),
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
    pub fn value_reference<M: MemoryView>(&self, memory: M) -> Option<Reference<T, M>> {
        (self.address > 0).then(|| Reference::new(memory, self.address))
    }
}

impl<T: SizedViewable> Ptr64<T> {
    /// Create a copy of the value the pointer points to
    #[must_use = "copied result must be used"]
    pub fn value_copy<M: MemoryView>(&self, memory: &M) -> Result<Option<Copy<T>>, M::AccessError> {
        (self.address > 0)
            .then(|| Copy::<T>::read_from_memory(memory, self.address))
            .transpose()
    }
}

impl<T> Array<T> for Ptr64<[T]> {
    fn start_address(&self) -> u64 {
        self.address
    }

    fn len(&self) -> Option<usize> {
        None
    }
}

impl<T: 'static> Deref for Ptr64<[T]> {
    type Target = dyn Array<T>;

    fn deref(&self) -> &Self::Target {
        self
    }
}

impl<T, const N: usize> Array<T> for Ptr64<[T; N]> {
    fn start_address(&self) -> u64 {
        self.address
    }

    fn len(&self) -> Option<usize> {
        Some(N)
    }
}

impl<T: 'static, const N: usize> Deref for Ptr64<[T; N]> {
    type Target = dyn Array<T>;

    fn deref(&self) -> &Self::Target {
        self
    }
}
