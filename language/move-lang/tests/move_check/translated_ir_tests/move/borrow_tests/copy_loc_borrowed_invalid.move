module 0x8675309::Tester {
    fun t() {
        let x = 0;
        let r1 = &mut x;
        copy x;
        r1;
    }
}

// check: COPYLOC_EXISTS_BORROW_ERROR
