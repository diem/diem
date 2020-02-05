module Tester {
    fun t() {
        let x = 0;
        let y = 0;
        let r1 = foo(&mut x, &mut y);
        copy x;
        r1;
    }

    fun foo(r: &mut u64, r2: &mut u64): &mut u64 {
        r2
    }
}

// check: COPYLOC_EXISTS_BORROW_ERROR
