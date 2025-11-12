use proc_macro::TokenStream;
use syn::parse_macro_input;

mod derive_raw_struct;

/// Marks a struct as a representation of a C-style struct with memory-mapped fields.
///
/// # Supported Attributes:  
/// - `size = "<struct size>"`  
///   Defines the total memory size of the struct.  
///   Structs attributed with size will implement the `SizedViewable` trait and be `Copy`able.  
///  
/// - `resolver = "my_resolver_fn"`  
///   Define a custom offset resolver where the raw field attribute value of `offset` will be passed into.
///   This allows a relaxiation of the `offset` value of the field as it may be anything. The function must return an u64.
///   By default the resolver is `core::convert::identity`.
///
/// Each field within the struct must be annotated with the `#[field(...)]` attribute.
///
/// # `#[field(...)]` Attributes:  
/// - `offset = "<field offset>"` (required)  
///   Specifies the memory offset of the field within the struct.
///   This can be either a fixed value (e.g., `0x08`) or a function call (e.g., `get_offset_field_a()`).  
///   **Note:** If a function call is used, the function will be executed each time the getter is invoked
///   to determine the field's offset.
///
/// # Example:
/// ```ignore
/// #[raw_struct(size = 0x10)]
/// struct MyStruct {
///     #[field(offset = 0x00)]
///     pub field_a: u32,
///
///     #[field(offset = 0x04)]
///     pub field_b: u32,
///
///     #[field(offset = 0x08)]
///     pub field_c: [u8; 0x8],
/// }
/// ```
#[proc_macro_attribute]
pub fn raw_struct(attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let attr = parse_macro_input!(attr);

    derive_raw_struct::raw_struct(attr, input)
        // .inspect(|result| println!("{}", result.to_string()))
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
