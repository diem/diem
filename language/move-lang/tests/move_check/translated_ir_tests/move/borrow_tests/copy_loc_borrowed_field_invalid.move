module 0x8675309::Tester {
    struct T has copy, drop { f: u64 }

    fun t() {
        let x = T { f: 0 };
        let r1 = &mut x.f;
        copy x;
        r1;
    }
}

// check: COPYLOC_EXISTS_BORROW_ERROR
