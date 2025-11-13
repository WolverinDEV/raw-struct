use core::{
    marker::PhantomData,
    mem,
};

use crate::CopyConstructable;

pub trait Viewable {
    fn name() -> &'static str;
    // fn fields() -> &'static [&'static dyn ViewableField];
}

pub trait ViewableSized: Viewable {
    type Memory: CopyConstructable;

    fn memory_size() -> usize {
        mem::size_of::<Self::Memory>()
    }
}

/// Declare that one type extends the other
/// ```rust
/// # use raw_struct::ViewableExtends;
///
/// struct C_BaseClass;
/// struct C_SubClass;
///
/// impl ViewableExtends<C_BaseClass> for C_SubClass {}
/// ```
pub trait ViewableExtends<T> {}

// Every type extends itself
impl<T> ViewableExtends<T> for T {}

pub trait ViewableField {
    fn name(&self) -> &'static str;
    fn offset(&self) -> u64;
}

pub struct TypedViewableField<V, T> {
    name: &'static str,
    offset_fn: &'static dyn Fn() -> u64,
    _type: PhantomData<(V, T)>,
}

impl<V, T> TypedViewableField<V, T> {
    pub const fn define(name: &'static str, offset_fn: &'static dyn Fn() -> u64) -> Self {
        Self {
            name,
            offset_fn,
            _type: PhantomData {},
        }
    }
}

impl<V, T> ViewableField for TypedViewableField<V, T> {
    fn name(&self) -> &'static str {
        self.name
    }

    fn offset(&self) -> u64 {
        (self.offset_fn)()
    }
}
