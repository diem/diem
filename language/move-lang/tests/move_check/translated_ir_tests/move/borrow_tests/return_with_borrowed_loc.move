module 0x8675309::M {
    struct X has copy, drop { y: Y }
    struct Y has copy, drop { u: u64 }

    fun t1() {
        let x = 0;
        let y = &x;
        copy y;
    }

    fun t2() {
        let s = X { y: Y { u: 0 } };
        let x = &s;
        let y = &x.y;
        let u = &y.u;
        copy x;
        copy y;
        copy u;
    }
}
