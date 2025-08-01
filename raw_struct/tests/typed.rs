use raw_struct::raw_struct;

#[raw_struct(size = 0x00)]
struct MyTypedStruct<T> {
    #[field(offset = 0x00)]
    pub field_a: u32,
}
