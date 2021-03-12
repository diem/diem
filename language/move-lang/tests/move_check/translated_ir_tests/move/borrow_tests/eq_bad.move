module 0x8675309::Tester {
    fun eqtest1() {
        let x: u64;
        let r: &mut u64;

        x = 0;
        r = &mut x;
        _ = copy r == copy r;
    }

    fun eqtest2() {
        let x: u64;
        let r: &mut u64;

        x = 0;
        r = &mut x;
        _ = copy r == move r;
    }

    fun neqtest1() {
        let x: u64;
        let r: &mut u64;

        x = 0;
        r = &mut x;
        _ = copy r != copy r;
    }

    fun neqtest2() {
        let x: u64;
        let r: &mut u64;

        x = 0;
        r = &mut x;
        _ = copy r != move r;
    }
}

// check: READREF_EXISTS_MUTABLE_BORROW_ERROR
// check: READREF_EXISTS_MUTABLE_BORROW_ERROR
// check: READREF_EXISTS_MUTABLE_BORROW_ERROR
// check: READREF_EXISTS_MUTABLE_BORROW_ERROR
