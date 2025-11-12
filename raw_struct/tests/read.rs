use raw_struct::{
    raw_struct,
    Copy,
};

#[raw_struct(size = 0x100)]
struct Dummy {
    #[field(offset = 0x00)]
    pub field_a: u32,

    #[field(offset = 0x04)]
    pub field_b: u32,

    #[field(offset = 0x08)]
    pub field_c: [u8; 0x08],
}

#[test]
fn test_field_read() {
    let mut memory = [0u8; 0x100];
    memory[0..4].copy_from_slice(&0xDEADBEEFu32.to_le_bytes());
    memory[4..8].copy_from_slice(&0x99u32.to_le_bytes());

    let object = Copy::<Dummy>::new(memory);

    assert_eq!(object.read_field(Dummy::field_a).unwrap(), 0xDEADBEEF);
    assert_eq!(object.read_field(Dummy::field_b).unwrap(), 0x99);
    assert_eq!(object.read_field(Dummy::field_c).unwrap(), [0u8; 0x08]);
}

#[test]
fn test_field_reference() {
    let mut memory = [0u8; 0x100];
    memory[0..4].copy_from_slice(&0xDEADBEEFu32.to_le_bytes());
    memory[4..8].copy_from_slice(&0x99u32.to_le_bytes());

    let object = Copy::<Dummy>::new(memory);
    let field_b = object.reference_field(Dummy::field_b);
    assert_eq!(field_b.read().unwrap(), 0x99);
}
