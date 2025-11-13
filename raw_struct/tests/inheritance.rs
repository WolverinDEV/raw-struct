use raw_struct::{
    raw_struct,
    Copy,
};

#[raw_struct(size = 0x10)]
struct BaseStruct {
    #[field(offset = 0x00)]
    value_a: u64,
}

#[raw_struct(size = 0x20, inherits = "BaseStruct")]
struct SubStruct {
    #[field(offset = 0x10)]
    value_b: u64,
}

#[test]
fn test_inheritence() {
    let mut memory = [0u8; 0x20];
    memory[0x00..0x08].copy_from_slice(&0xB00B5B00B5u64.to_le_bytes());
    memory[0x10..0x18].copy_from_slice(&0xDEADBEEFu64.to_le_bytes());

    let object = Copy::<SubStruct>::new(memory);
    assert_eq!(object.read_field(BaseStruct::value_a), Ok(0xB00B5B00B5));
    assert_eq!(object.read_field(SubStruct::value_b), Ok(0xDEADBEEF));
}
