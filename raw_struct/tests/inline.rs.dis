use raw_struct::{
    raw_struct,
    Copy,
};

#[raw_struct(size = 0x08)]
struct B {
    #[field(offset = 0x00)]
    value: u64,
}

#[raw_struct(size = 0x10)]
struct A {
    #[field(offset = 0x08)]
    val_b: Copy<B>,
}

#[test]
fn test_inline() {
    let mut memory = [0u8; 0x10];
    memory[8..12].copy_from_slice(&0xDEADBEEFu32.to_le_bytes());

    let object = Copy::<A>::new(memory);
    assert_eq!(object.val_b().unwrap().value().unwrap(), 0xDEADBEEF);
}
