use raw_struct::raw_struct;

#[test]
fn test_getter_rename() {
    #[raw_struct(size = 0x08)]
    struct Dummy {
        #[field(offset = 0x00)]
        field_a: u64,

        #[field(0x08)]
        field_b: u64,
    }

    // #[raw_struct(size = 0x08, resolver = "my_dummy_function")]
    // struct Dummy2 {
    //     #[field(0x00)]
    //     field_my_flag: bool,
    // }

    // let instance = Copy::<Dummy>::new([0x00; 0x08]);
    // instance.get_field_d().unwrap();
}
