module Tester {
    struct T { f: u64 }

    fun t() {
        let x = T { f: 0 };
        let r1 = &x.f;
        let r2 = &x.f;
        copy x;
        r1;
        r2;
    }
}
