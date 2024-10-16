use alloc::{
    borrow::Cow,
    boxed::Box,
    format,
};
use core::{
    error::Error,
    fmt,
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

impl Error for AccessViolation {}

#[derive(Debug)]
pub struct AccessError {
    pub source: Box<dyn Error + 'static>,

    pub offset: u64,
    pub size: usize,
    pub mode: AccessMode,

    pub object: Cow<'static, str>,
    pub member: Option<&'static str>,
}

impl fmt::Display for AccessError {
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

impl Error for AccessError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.source)
    }
}
