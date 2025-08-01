#![cfg_attr(feature = "no_std", no_std)]

pub mod builtins;

mod error;
pub use error::{
    MemoryDecodeError,
    OutOfBoundsViolation,
};

mod memory;
pub use memory::{
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

extern crate alloc;
pub use alloc::borrow::Cow;
