use std::{
    self,
    error::Error,
    marker,
};

use raw_struct::{
    raw_struct,
    Copy,
    CopyMemory,
    FromMemoryView,
    SizedViewable,
};

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let buffer = CopyMemory([0x1122u64, 0x8877, 0x9988]);
    let object = Copy::<Container<u64>>::read_from_memory(&buffer, 0x00)?;

    println!(
        "Memory size: 0x{:X}",
        <Container::<u64> as SizedViewable>::memory_size()
    );
    println!("Vat a = 0x{:X}", object.var_a()?);
    println!("Inner = 0x{:X}", object.inner()?);
    println!("Vat b = 0x{:X}", object.var_b()?);
    Ok(())
}

#[raw_struct(memory = "([u8; 0x10], T)")]
struct Container<T: marker::Copy + FromMemoryView + Send + Sync + 'static> {
    #[field(offset = 0x00)]
    pub var_a: u64,

    #[field(offset = 0x08)]
    pub inner: T,

    #[field(offset = "0x08 + core::mem::size_of::<T>()")]
    pub var_b: u64,
}
