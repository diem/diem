module M {
    struct S { f: u64, g: bool }

    fun t0() {
        let x;
    }

    fun t1() {
        let (x, y);
    }

    fun t2() {
        let S{ f, g };
    }
}
