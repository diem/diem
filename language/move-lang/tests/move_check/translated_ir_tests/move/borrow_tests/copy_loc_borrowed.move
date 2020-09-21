module Tester {
    fun t() {
        let x = 0;
        let r1 = &x;
        let r2 = &x;
        x + copy x;
        r1;
        r2;
    }
}
