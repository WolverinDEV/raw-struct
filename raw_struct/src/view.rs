use alloc::borrow::Cow;

use crate::memory::MemoryView;

pub trait ViewableBase: Send + Sync {
    fn object_memory(&self) -> &dyn MemoryView;
}

pub trait ViewableImplementation<M: MemoryView, T: ?Sized>: ViewableBase {
    fn memory(&self) -> &M;
    fn as_trait(&self) -> &T;
}

pub trait Viewable<T: ?Sized>: 'static {
    type Memory: Copy + Send + Sync;
    type Implementation<M: MemoryView + 'static>: ViewableImplementation<M, T>;

    const MEMORY_SIZE: usize = core::mem::size_of::<Self::Memory>();

    fn create<M: MemoryView + 'static>(memory: M) -> Self::Implementation<M>;
    fn name() -> Cow<'static, str>;
}
