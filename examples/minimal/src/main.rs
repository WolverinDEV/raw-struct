use std::{
    self,
    error::Error,
    sync::Arc,
};

use raw_struct::{
    raw_struct,
    Copy,
    Reference,
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut memory = [0u8; 0x10];
    memory[0..4].copy_from_slice(&0x6Fu32.to_le_bytes());
    memory[4..8].copy_from_slice(&0x99u32.to_le_bytes());

    {
        let object = Reference::<dyn MyStruct>::new(0x00, Arc::new(memory.clone()));
        println!("field_a = {}", object.field_a()?);
        println!("field_b = {}", object.field_b()?);
    }

    {
        let object = Copy::<dyn MyStruct>::new(memory);
        println!("field_a = {}", object.field_a()?);
        println!("field_b = {}", object.field_b()?);
    }

    Ok(())
}

#[raw_struct(size = 0x10)]
struct MyStruct {
    #[field(offset = 0x00)]
    pub field_a: u32,

    #[field(offset = 0x04)]
    pub field_b: u32,

    #[field(offset = 0x08, getter = "get_field_c")]
    pub field_c: [u8; 0x8],
}
