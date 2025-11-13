use raw_struct::{
    builtins::Ptr64,
    raw_struct,
    MemoryView,
    MemoryViewDereferenceable,
    Reference,
};

#[test]
fn test_memory_dereference() {
    #[derive(Debug, Clone, Copy)]
    struct IdentityMemoryView<M: MemoryView>(M);

    impl<M: MemoryView> MemoryView for IdentityMemoryView<M> {
        type AccessError = M::AccessError;

        fn read_memory(&self, offset: u64, buffer: &mut [u8]) -> Result<(), Self::AccessError> {
            self.0.read_memory(offset, buffer)
        }
    }

    impl<M: MemoryView> MemoryViewDereferenceable for IdentityMemoryView<M> {
        fn dereference(&self, address: u64) -> Result<u64, Self::AccessError> {
            Ok(address)
        }
    }

    let mut memory = [0u8; 0x44];
    memory[0..4].copy_from_slice(&0x6Fu32.to_le_bytes());
    memory[4..8].copy_from_slice(&0x99u32.to_le_bytes());
    memory[4..8].copy_from_slice(&0x99u32.to_le_bytes());
    memory[0x40..][..4].copy_from_slice(&0xAAu32.to_le_bytes());

    let object = Reference::<MyStruct, _>::new(IdentityMemoryView(memory.as_slice()), 0x00);
    let field_d = object.dereference_field(MyStruct::field_e).unwrap();
    for index in 0..0x10 {
        let value = field_d.read_element(index).unwrap();
        println!("Read value = {value}");
    }
}

#[raw_struct(size = 0x40)]
struct MyStruct {
    /// Array to another copyable
    #[field(offset = 0x10)]
    pub field_e: Ptr64<[u8]>,
}
