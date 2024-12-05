use alloc::borrow::Cow;

use crate::memory::MemoryView;

pub trait ViewableBase<M: MemoryView>: Send + Sync {
    fn object_memory(&self) -> &M;
}

pub trait ViewableInstance<A: ?Sized, M: MemoryView>: ViewableBase<M> {
    fn get_accessor(&self) -> &A;
}

pub trait Viewable: 'static {
    // TODO: Move this into Copyable
    /// Const memory used for copying the value
    type Memory: Copy + Send + Sync;

    /// Accessor trait used to access memory contents
    type Accessor<M: MemoryView + 'static>: ?Sized;

    /// View instance type packed by memory view M
    type Instance<M: MemoryView + 'static>: ViewableInstance<Self::Accessor<M>, M>;

    // TODO: Move this into Copyable
    /// Byte size of the copy memory
    const MEMORY_SIZE: usize = core::mem::size_of::<Self::Memory>();

    /// Debug name of the Viewable
    fn name() -> Cow<'static, str>;

    /// Create a new view backed by the given memory
    fn create_view<M: MemoryView + 'static>(memory: M) -> Self::Instance<M>;
}
