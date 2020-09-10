module Tester {
    fun eqtest1() {
        let x: u64;
        let r: &mut u64;

        x = 0;
        r = &mut x;
        _ = freeze(copy r) == freeze(copy r);
    }

    fun eqtest2() {
        let x: u64;
        let r: &mut u64;

        x = 0;
        r = &mut x;
        _ = freeze(copy r) == freeze(move r);
    }

    fun neqtest1() {
        let x: u64;
        let r: &mut u64;

        x = 0;
        r = &mut x;
        _ = freeze(copy r) != freeze(copy r);
    }

    fun neqtest2() {
        let x: u64;
        let r: &mut u64;

        x = 0;
        r = &mut x;
        _ = freeze(copy r) != freeze(move r);
    }
}
