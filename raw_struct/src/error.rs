use core::fmt::{
    self,
    Debug,
    Display,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct OutOfBoundsViolation {
    pub access_offset: usize,
    pub access_len: usize,

    pub src_len: usize,
}

impl fmt::Display for OutOfBoundsViolation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "memory outside of the struct has been accessed")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for OutOfBoundsViolation {}

#[cfg(not(feature = "std"))]
impl core::error::Error for OutOfBoundsViolation {}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum MemoryDecodeError<A, V> {
    MemoryAccess(A),
    ValueDecode(V),
}

impl<A: Display, V: Display> fmt::Display for MemoryDecodeError<A, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MemoryAccess(inner) => inner.fmt(f),
            Self::ValueDecode(inner) => inner.fmt(f),
        }
    }
}

#[cfg(feature = "std")]
impl<A: Display + Debug, V: Display + Debug> std::error::Error for MemoryDecodeError<A, V> {}

#[cfg(not(feature = "std"))]
impl<A: Display + Debug, V: Display + Debug> core::error::Error for MemoryDecodeError<A, V> {}
