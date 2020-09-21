module M {
    fun t1() {
        let a = 0;
        let r1 = &mut a;
        let r2 = &mut a;
        *r2 = 2;
        *r1 = 1;
    }
}

// check: WRITEREF_EXISTS_BORROW_ERROR
