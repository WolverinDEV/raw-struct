use core::mem;

use crate::MemoryView;

pub trait ViewableImplementation<M> {
    fn memory_view(&self) -> &M;
    fn into_memory_view(self) -> M;
}

pub trait Viewable {
    type Implementation<M: MemoryView>: ViewableImplementation<M>;

    fn name() -> &'static str;
    fn from_memory<M: MemoryView>(memory: M) -> Self::Implementation<M>;
}

pub trait SizedViewable: Viewable {
    type Memory: Clone + Copy + Send + Sync;

    fn memory_size() -> usize {
        mem::size_of::<Self::Memory>()
    }
}
