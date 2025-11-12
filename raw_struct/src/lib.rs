#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(rustfmt, rustfmt_skip)]

#[cfg(feature = "alloc")]
extern crate alloc;

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

mod reference;
pub use reference::Reference;

mod copy;
pub use copy::{
    Copy,
    CopyMemory,
};

mod view;
pub use view::{
    Viewable,
    ViewableExtends,
    ViewableField,
    ViewableSized,
};

pub mod builtins;

// Re-exports
pub use raw_struct_derive::raw_struct;