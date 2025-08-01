use raw_struct::{
    raw_struct,
    Copy,
};

#[test]
fn test_getter_rename() {
    #[raw_struct(size = 0x08)]
    struct Dummy {
        #[field(offset = 0x00, getter = "get_field_d")]
        field_a: u64,
    }

    let instance = Copy::<Dummy>::new([0x00; 0x08]);
    instance.get_field_d().unwrap();
}
