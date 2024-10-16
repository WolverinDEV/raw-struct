use std::sync::atomic::{
    AtomicBool,
    Ordering,
};

use raw_struct::{
    raw_struct,
    Copy,
};

#[raw_struct(size = 0x10)]
struct Dummy {
    #[field(offset = "get_offset()")]
    field_01: u64,
}

static FN_CALLED: AtomicBool = AtomicBool::new(false);
fn get_offset() -> u64 {
    assert!(!FN_CALLED.load(Ordering::Relaxed));
    FN_CALLED.store(true, Ordering::Relaxed);

    0x00
}

#[test]
fn test() {
    assert!(!FN_CALLED.load(Ordering::Relaxed));
    let value = Copy::<dyn Dummy>::new([0x0; 0x10]);
    value.field_01().unwrap();
    assert!(FN_CALLED.load(Ordering::Relaxed));
}
