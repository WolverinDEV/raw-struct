use alloc::sync::Arc;
use core::{
    marker::{
        self,
        PhantomData,
    },
    mem,
    ops::Deref,
};

use super::{
    array::SizedArray,
    Array,
};
use crate::{
    AccessError,
    AccessMode,
    Copy,
    FromMemoryView,
    MemoryView,
    Reference,
    Viewable,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ptr64<T>
where
    T: 'static + ?Sized,
{
    pub address: u64,
    _dummy: PhantomData<T>,
}

impl<T: ?Sized + 'static> Clone for Ptr64<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: ?Sized + 'static> marker::Copy for Ptr64<T> {}

impl<T: ?Sized> Ptr64<T> {
    pub fn is_null(&self) -> bool {
        self.address == 0
    }

    pub fn cast<V: ?Sized>(&self) -> Ptr64<V> {
        Ptr64::<V> {
            address: self.address,
            _dummy: Default::default(),
        }
    }
}

impl<T: marker::Copy> Ptr64<T> {
    /// Create a copy of the value the pointer points to
    #[must_use = "copied result must be used"]
    pub fn read_value(&self, memory: &dyn MemoryView) -> Result<Option<T>, AccessError> {
        if self.address > 0 {
            let memory = T::read_object(memory, self.address).map_err(|err| AccessError {
                source: err,

                member: None,
                object: "T".into(),
                mode: AccessMode::Read,

                offset: self.address,
                size: mem::size_of::<T>(),
            })?;

            Ok(Some(memory))
        } else {
            Ok(None)
        }
    }
}

impl<T: ?Sized + Viewable<T>> Ptr64<T> {
    #[must_use]
    pub fn value_reference(&self, memory: Arc<dyn MemoryView>) -> Option<Reference<T>> {
        if self.address > 0 {
            Some(Reference::new(memory, self.address))
        } else {
            None
        }
    }

    /// Create a copy of the value the pointer points to
    #[must_use = "copied result must be used"]
    pub fn value_copy(&self, memory: &dyn MemoryView) -> Result<Option<Copy<T>>, AccessError> {
        if self.address > 0 {
            let memory =
                T::Memory::read_object(memory, self.address).map_err(|err| AccessError {
                    source: err,

                    member: None,
                    object: T::name(),
                    mode: AccessMode::Read,

                    offset: self.address,
                    size: mem::size_of::<T::Memory>(),
                })?;

            Ok(Some(Copy::new(memory)))
        } else {
            Ok(None)
        }
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

impl<T> Deref for Ptr64<[T]> {
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

impl<T, const N: usize> Deref for Ptr64<[T; N]> {
    type Target = dyn Array<T>;

    fn deref(&self) -> &Self::Target {
        self
    }
}

impl<T: ?Sized> Array<T> for Ptr64<dyn Array<T>> {
    fn start_address(&self) -> u64 {
        self.address
    }

    fn len(&self) -> Option<usize> {
        None
    }
}

impl<T: ?Sized> Deref for Ptr64<dyn Array<T>> {
    type Target = dyn Array<T>;

    fn deref(&self) -> &Self::Target {
        self
    }
}

impl<T: ?Sized, const N: usize> Array<T> for Ptr64<dyn SizedArray<T, N>> {
    fn start_address(&self) -> u64 {
        self.address
    }

    fn len(&self) -> Option<usize> {
        Some(N)
    }
}

impl<T: ?Sized, const N: usize> Deref for Ptr64<dyn SizedArray<T, N>> {
    type Target = dyn Array<T>;

    fn deref(&self) -> &Self::Target {
        self
    }
}
