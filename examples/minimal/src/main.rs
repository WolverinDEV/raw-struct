use std::{
    self,
    error::Error,
    sync::Arc,
};

use raw_struct::{
    builtins::{
        Array,
        Ptr64,
        SizedArray,
    },
    raw_struct,
    Copy,
    FromMemoryView,
    Reference,
    Viewable,
};

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut memory = [0u8; 0x20];
    memory[0..4].copy_from_slice(&0x6Fu32.to_le_bytes());
    memory[4..8].copy_from_slice(&0x99u32.to_le_bytes());

    println!(
        "{}",
        <dyn MyStruct as Viewable::<dyn MyStruct>>::MEMORY_SIZE
    );

    let memory = Arc::new(memory);
    {
        let object = Reference::<dyn MyStruct>::new(memory.clone(), 0x00);
        println!("field_a = {}", object.field_a()?);
        println!("field_b = {}", object.field_b()?);
    }

    {
        let object = Copy::<dyn MyStruct>::read_object(&*memory, 0x00)?;
        println!("field_a = {}", object.field_a()?);
        println!("field_b = {}", object.field_b()?);
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

    /// Showcasing the custom getter name
    #[field(offset = 0x08, getter = "get_field_c")]
    pub field_c: [u8; 0x8],

    /// Sized array of other raw_structs
    #[field(offset = 0x10)]
    pub field_d: Ptr64<dyn SizedArray<dyn MyArrayElement, 0x20>>,

    /// Array to another copyable
    #[field(offset = 0x10)]
    pub field_e: Ptr64<[u8]>,

    /// Advanced array to other raw_structs
    #[field(offset = 0x18)]
    pub field_f: Ptr64<dyn Array<dyn MyStruct>>,

    /// Advanced array to other raw_structs
    #[field(offset = 0x18)]
    pub field_fb: Ptr64<dyn Array<u64>>,

    #[field(offset = 0x20)]
    pub field_g: [u8; 0x20],
}

#[raw_struct(size = 0x44)]
struct MyStructExt {
    #[field(offset = 0x40)]
    pub ext_field_a: u32,
}
impl MyStruct for dyn MyStructExt {}
