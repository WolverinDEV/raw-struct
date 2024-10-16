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
};
pub use memory::{
    FromMemoryView,
    MemoryView,
};
pub use raw_struct_derive::raw_struct;
pub use reference::Reference;
pub use view::{
    Viewable,
    ViewableBase,
    ViewableImplementation,
};

extern crate alloc;
pub use alloc::borrow::Cow;
