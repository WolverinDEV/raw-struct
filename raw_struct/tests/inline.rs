use std::sync::Arc;

use raw_struct::{
    raw_struct,
    Copy,
    MemoryView,
    Reference,
};

#[raw_struct(size = 0x08)]
struct B {
    #[field(offset = 0x00)]
    value: u64,
}

#[raw_struct(size = 0x10)]
struct A {
    #[field(offset = 0x08)]
    val_b: Copy<dyn B>,
}

#[test]
fn test_inline() {
    let mut memory = [0u8; 0x10];
    memory[8..12].copy_from_slice(&0xDEADBEEFu32.to_le_bytes());

    let object = Copy::<dyn A>::new(memory);
    assert_eq!(object.val_b().unwrap().value().unwrap(), 0xDEADBEEF);

    let memory_view: Arc<dyn MemoryView<Error = _>> = Arc::new(memory);
    let _object = Reference::<dyn A, _>::new(memory_view, 0x00);
}
