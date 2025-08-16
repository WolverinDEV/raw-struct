#![feature(new_range_api)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(all(feature = "alloc", not(feature = "std")))]
pub(crate) use core::range::Range;
#[cfg(all(feature = "std"))]
pub(crate) use std::range::Range;

pub mod builtins;

mod error;
pub use error::{
    MemoryDecodeError,
    OutOfBoundsViolation,
};

mod memory;
pub use memory::{
    CopyConstructable,
    FromMemoryView,
    MemoryView,
};

mod view;
pub use view::{
    SizedViewable,
    Viewable,
    ViewableImplementation,
};

mod reference;
pub use reference::{
    Reference,
    ReferenceMemory,
};

mod copy;

pub use copy::{
    Copy,
    CopyMemory,
};
pub use raw_struct_derive::raw_struct;
