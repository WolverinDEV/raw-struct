use raw_struct::{
    raw_struct,
    Viewable,
};

#[test]
fn test_struct_metadata() {
    #[raw_struct]
    struct MyStruct {
        #[field(0x00)]
        pub FieldA: u32,

        #[field(0x08)]
        pub FieldB: u32,

        #[field(0xEB)]
        pub FieldX: u8,
    }

    assert_eq!(MyStruct::name(), "MyStruct");

    let fields = MyStruct::fields()
        .iter()
        .map(|field| format!("{:X}->{}", field.offset(), field.name()))
        .collect::<Vec<_>>();

    assert_eq!(fields, &["0->FieldA", "8->FieldB", "EB->FieldX"]);
}
