use std::{
    marker::PhantomData,
    mem,
    ops::Deref,
};

use raw_struct::{
    Copy,
    MemoryView,
    Reference,
    SizedViewable,
    Viewable,
    ViewableImplementation,
};

struct MyParent<M = ()>(M);

impl<M: MemoryView> MyParent<M> {
    pub fn say_hi_parent(&self) {}
}

impl<M: MemoryView> ViewableImplementation<M> for MyParent<M> {
    fn memory_view(&self) -> &M {
        &self.0
    }

    fn into_memory_view(self) -> M {
        self.0
    }
}

impl Viewable for MyParent {
    type Implementation<M: MemoryView> = MyParent<M>;

    fn name() -> &'static str {
        "MyParent"
    }

    fn from_memory<M: MemoryView>(memory: M) -> Self::Implementation<M> {
        MyParent(memory)
    }
}

struct MyStruct<M = ()>(MyParent<M>);

impl<M: MemoryView> MyStruct<M> {
    pub fn say_hi(&self) {}
}

impl<M: MemoryView> ViewableImplementation<M> for MyStruct<M> {
    fn memory_view(&self) -> &M {
        self.0.memory_view()
    }

    fn into_memory_view(self) -> M {
        self.0.into_memory_view()
    }
}

impl Viewable for MyStruct {
    type Implementation<M: MemoryView> = MyStruct<M>;

    fn name() -> &'static str {
        "MyStruct"
    }

    fn from_memory<M: MemoryView>(memory: M) -> Self::Implementation<M> {
        MyStruct(MyParent::from_memory(memory))
    }
}

impl<M: MemoryView> Deref for MyStruct<M> {
    type Target = MyParent<M>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SizedViewable for MyStruct {
    type Memory = [u8; 0x08];
}

struct MyTypedStruct<T, M = ()> {
    memory: M,
    _type: PhantomData<(T,)>,
}

impl<T, M: MemoryView> MyTypedStruct<T, M> {
    pub fn get_type_size(&self) -> usize {
        mem::size_of::<T>()
    }
}

impl<T, M: MemoryView> ViewableImplementation<M> for MyTypedStruct<T, M> {
    fn memory_view(&self) -> &M {
        &self.memory
    }

    fn into_memory_view(self) -> M {
        self.memory
    }
}

impl<T> Viewable for MyTypedStruct<T> {
    type Implementation<M: MemoryView> = MyTypedStruct<T, M>;

    fn name() -> &'static str {
        "MyTypedStruct<T>"
    }

    fn from_memory<M: MemoryView>(memory: M) -> Self::Implementation<M> {
        MyTypedStruct {
            memory,
            _type: Default::default(),
        }
    }
}

#[test]
fn raw_dummy() {
    let memory = &[0x00u8, 0x00, 0x00, 0x00];
    let reference = Reference::<MyStruct, _>::new(memory.as_slice(), 0x00);
    reference.say_hi();
    reference.say_hi_parent();

    let _copy = unsafe { Copy::<MyStruct>::new_zerod() };

    let typed = reference.cast::<MyTypedStruct<u32>>();
    println!("Typed size: {}", typed.get_type_size());
}

// fn take_reference(value: &dyn MyStruct) {}
