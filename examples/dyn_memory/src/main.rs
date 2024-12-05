use std::{
    self,
    error::Error,
    marker::{
        self,
    },
};

use raw_struct::{
    raw_struct,
    Copy,
    FromMemoryView,
    Viewable,
};

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    /*
     * Note:
     * Accessing all the container entries evaluates in release mode to just a mov instruction.
     */
    let buffer = [0x1122u64, 0x8877, 0x9988];
    let object = Copy::<dyn Container<u64>>::read_object(&buffer, 0x00)?;

    println!(
        "Memory size: 0x{:X}",
        <dyn Container::<u64> as Viewable>::MEMORY_SIZE
    );
    println!("Vat a = 0x{:X}", object.var_a()?);
    println!("Inner = 0x{:X}", object.inner()?);
    println!("Vat b = 0x{:X}", object.var_b()?);
    Ok(())
}

#[raw_struct(memory = "([u8; 0x10], T)")]
struct Container<T>
where
    T: marker::Copy + Send + Sync + 'static,
{
    #[field(offset = 0x00)]
    pub var_a: u64,

    #[field(offset = 0x08)]
    pub inner: T,

    #[field(offset = "0x08 + core::mem::size_of::<T>()")]
    pub var_b: u64,
}
