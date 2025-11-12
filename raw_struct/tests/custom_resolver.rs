use raw_struct::{
    raw_struct,
    Reference,
};

#[test]
fn test_custom_resolver() {
    #[allow(dead_code)]
    enum Offset {
        FieldA,
        FieldB,
    }

    impl Offset {
        fn resolve(self) -> u64 {
            match self {
                Self::FieldA => 0x00,
                Self::FieldB => 0x01,
            }
        }
    }

    #[raw_struct(resolver = "Offset::resolve")]
    struct Dummy {
        #[field(Offset::FieldA)]
        field_a: u8,

        #[field(Offset::FieldB)]
        field_b: u8,
    }

    let data = [0xFFu8, 0x10].as_slice();
    let value = Reference::<Dummy, _>::new(&data, 0x00);
    assert_eq!(value.read_field(Dummy::field_b), Ok(0x10));
}
