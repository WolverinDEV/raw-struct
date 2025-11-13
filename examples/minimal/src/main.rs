use std::{
    self,
    error::Error,
};

use raw_struct::{
    builtins::Ptr64,
    raw_struct,
    Copy,
    FromMemoryView,
    Reference,
    ViewableSized,
};

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut memory = [0u8; 0x44];
    memory[0..4].copy_from_slice(&0x6Fu32.to_le_bytes());
    memory[4..8].copy_from_slice(&0x99u32.to_le_bytes());
    memory[4..8].copy_from_slice(&0x99u32.to_le_bytes());
    memory[0x40..][..4].copy_from_slice(&0xAAu32.to_le_bytes());

    println!(
        "MyStruct size = 0x{:X}",
        <MyStruct as ViewableSized>::memory_size()
    );

    {
        let object = Reference::<MyStruct, _>::new(memory.as_slice(), 0x00);
        println!("field_a = {:X}", object.read_field(MyStruct::field_a)?);
        println!("field_b = {:X}", object.read_field(MyStruct::field_b)?);

        let object = object.cast::<MyStructExt>();
        println!(
            "ext_field_a = {:X}",
            object.read_field(MyStructExt::ext_field_a)?
        );
        println!(
            "field_base = {:X}",
            object.read_field(MyStructExt::ext_field_a)?
        );
    }

    {
        let object = Copy::<MyStruct>::read_from_memory(&memory.as_slice(), 0x00)?;
        println!("field_a = {:X}", object.read_field(MyStruct::field_a)?);
        println!("field_b = {:X}", object.read_field(MyStruct::field_b)?);
    }

    Ok(())
}

#[raw_struct(size = 0x08)]
struct MyArrayElement {}

#[raw_struct(size = 0x40)]
struct MyStruct {
    /// u32 field located at offset 0
    #[field(offset = 0x00)]
    pub field_a: u32,

    /// u32 field located at offset 4
    #[field(offset = 0x04)]
    pub field_b: u32,

    #[field(offset = 0x08)]
    pub field_c: [u8; 0x8],

    /// Sized array of other raw_structs
    #[field(offset = 0x10)]
    pub field_d: Ptr64<[MyArrayElement; 0x20]>,

    /// Array to another copyable
    #[field(offset = 0x10)]
    pub field_e: Ptr64<[u8]>,

    /// Advanced array to other raw_structs
    #[field(offset = 0x18)]
    pub field_f: Ptr64<[MyStruct]>,

    /// Advanced array to other raw_structs
    #[field(offset = 0x18)]
    pub field_fb: Ptr64<[u64]>,

    #[field(offset = 0x20)]
    pub field_g: [u8; 0x20],
}

#[raw_struct]
struct MyStructBase<T: FromMemoryView + 'static> {
    #[field(offset = 0x00)]
    pub field_base: T,
}

#[raw_struct(size = 0x44, inherits = "MyStructBase::<u32>")]
struct MyStructExt {
    #[field(offset = 0x40)]
    pub ext_field_a: u32,
}
