# raw_struct &emsp; [![Latest Version]][crates.io] [![License: GPL v3]](./LICENSE) [![GitHub build status]][actions]

[License: GPL v3]: https://img.shields.io/badge/License-GPLv3-blue.svg
[Latest Version]: https://img.shields.io/crates/v/raw_struct.svg
[crates.io]: https://crates.io/crates/raw_struct
[GitHub build status]: https://github.com/WolverinDEV/raw-struct/workflows/Rust/badge.svg?branch=master
[actions]: https://github.com/WolverinDEV/raw-struct/actions?query=workflow%3ARust

`raw_struct` is a Rust procedural macro for easily declaring C-style structs that reference local or external memory, based on your memory implementation. It generates appropiate getter methods for easy access. This crate has support for `no_std` environments.

## Usage
To use `raw_struct`, simply define a struct with the raw_struct attribute as following:
```rust
#[raw_struct(size = 0x10)]
struct MyStruct {
    #[field(offset = 0x00)]
    pub field_a: u32,

    #[field(offset = 0x04)]
    pub field_b: u32,

    #[field(offset = 0x08, getter = "get_field_c")]
    pub field_c: [u8; 0x8],
}
```

To reference the declared struct in memory you can ether do so by `Reference` or `Copy`:
```rust
let memory = [0u8; 0x10];
{
    let object = Reference::<MyStruct>::new(0x00, Arc::new(memory.clone()));
    println!("field_a = {}", object.field_a()?);
    println!("field_b = {}", object.field_b()?);
}

{
    let object = Copy::<MyStruct>::new(memory);
    println!("field_a = {}", object.field_a()?);
    println!("field_b = {}", object.field_b()?);
}
```

## Examples
Examples can be found within the examples directory of this repository. These examples demonstrate how to use raw_struct in various contexts.

To run the examples, clone the repository and use the following command:
```bash
cargo run --bin <example_name>
```