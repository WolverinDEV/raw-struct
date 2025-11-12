use core::{
    convert::Infallible,
    marker::PhantomData,
};

use crate::{
    FromMemoryView,
    MemoryDecodeError,
    MemoryView,
};

fn x() {
    // raw-struct types
    struct MemberField<S, T> {
        pub name: &'static str,
        pub offset: u64,
        _type: PhantomData<(S, T)>,
    }

    impl<S, T> MemberField<S, T> {
        pub const fn define(name: &'static str, offset: u64) -> Self {
            Self {
                name,
                offset,
                _type: PhantomData {},
            }
        }
    }

    trait RawStruct {
        fn struct_name() -> &'static str;
    }

    trait Extends<S> {}

    // Every type extends itself
    impl<T> Extends<T> for T {}

    // Helper types
    struct Ptr64<T> {
        address: u64,
        _target: PhantomData<T>,
    }

    impl<T> FromMemoryView for Ptr64<T> {
        type DecodeError = Infallible;

        fn read_object<M: MemoryView>(
            view: &M,
            offset: u64,
        ) -> Result<Self, MemoryDecodeError<M::AccessError, Self::DecodeError>> {
            Ok(Self {
                address: u64::read_object(view, offset)?,
                _target: Default::default(),
            })
        }
    }

    impl<T: RawStruct> Ptr64<T> {
        pub const fn new(address: u64) -> Self {
            Self {
                address,
                _target: PhantomData {},
            }
        }

        pub fn read_field<R: FromMemoryView, M: MemoryView, C>(
            &self,
            memory: &M,
            field: &MemberField<C, R>,
        ) -> Result<R, MemoryDecodeError<M::AccessError, R::DecodeError>>
        where
            T: Extends<C>,
        {
            R::read_object(memory, self.address + field.offset)
        }
    }

    // Test env
    #[allow(non_camel_case_types)]
    struct C_BaseClass {}

    impl RawStruct for C_BaseClass {
        fn struct_name() -> &'static str {
            "C_BaseClass"
        }
    }

    #[allow(non_upper_case_globals, unused)]
    impl C_BaseClass {
        pub const ValueA: &MemberField<Self, Ptr64<()>> = &MemberField::define("ValueA", 0x00);
    }

    #[allow(non_camel_case_types)]
    struct C_SubClass {}

    impl RawStruct for C_SubClass {
        fn struct_name() -> &'static str {
            "C_SubClass"
        }
    }

    impl Extends<C_BaseClass> for C_SubClass {}

    #[allow(non_upper_case_globals, unused)]
    impl C_SubClass {
        pub const ValueB: &MemberField<Self, Ptr64<()>> = &MemberField::define("ValueB", 0x04);
    }

    let memory = [0u8; 0x10];

    let instance = Ptr64::<C_SubClass>::new(0x00);
    let value = instance.read_field(&&memory[..], C_BaseClass::ValueA);
    let value = instance.read_field(&&memory[..], C_SubClass::ValueB);

    // Copy<C_SubClass>::from_memory(&&memory[..]).unwrap();
    // Reference<C_SubClass, CopiedMemory>
}
