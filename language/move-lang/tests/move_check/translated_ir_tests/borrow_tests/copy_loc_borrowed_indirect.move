module Tester {
    fun t() {
        let x = 0;
        let y = 0;
        let r1 = foo(&x, &y);
        let r2 = foo(&x, &y);
        x + copy x;
        y + copy y;
        r1;
        r2;
    }

    fun foo(r: &u64, r2: &u64): &u64 {
        r2
    }
}
