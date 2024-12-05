use alloc::{
    borrow::Cow,
    format,
};
use core::{
    convert::Infallible,
    fmt::{
        self,
        Debug,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AccessMode {
    Read,
    Write,
}

impl AccessMode {
    fn name(&self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AccessViolation;

impl fmt::Display for AccessViolation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "memory outside of the struct has been accessed")
    }
}

#[cfg(feature = "no_std")]
impl core::error::Error for AccessViolation {}

#[cfg(not(feature = "no_std"))]
impl std::error::Error for AccessViolation {}

#[derive(Debug)]
pub struct AccessError<S = Infallible> {
    pub source: S,

    pub offset: u64,
    pub size: usize,
    pub mode: AccessMode,

    pub object: Cow<'static, str>,
    pub member: Option<Cow<'static, str>>,
}

impl<S: fmt::Display> fmt::Display for AccessError<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "failed to {} 0x{:X} bytes at 0x{:X} ({}): {}",
            self.mode.name(),
            self.size,
            self.offset,
            if let Some(member) = &self.member {
                Cow::from(format!("{}::{}", self.object, member))
            } else {
                self.object.clone()
            },
            self.source
        )
    }
}

#[cfg(feature = "no_std")]
impl<S: core::error::Error + 'static> core::error::Error for AccessError<S> {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        Some(&self.source)
    }
}

#[cfg(not(feature = "no_std"))]
impl<S: std::error::Error + 'static> std::error::Error for AccessError<S> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}
