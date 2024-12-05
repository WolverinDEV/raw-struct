#![cfg_attr(feature = "no_std", no_std)]

pub mod builtins;
mod copy;
mod error;
mod memory;
mod reference;
mod view;

pub use copy::Copy;
pub use error::{
    AccessError,
    AccessMode,
    AccessViolation,
};
pub use memory::{
    CopyMemoryView,
    FromMemoryView,
    MemoryView,
};
pub use raw_struct_derive::raw_struct;
pub use reference::Reference;
pub use view::{
    Copyable,
    Viewable,
    ViewableBase,
    ViewableInstance,
};

extern crate alloc;
pub use alloc::borrow::Cow;
