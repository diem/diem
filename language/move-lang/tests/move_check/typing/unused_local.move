module M {
    struct S { f: u64, g: bool }

    fun t0() {
        let x: u64;
    }

    fun t1() {
        let (x, y): (u64, u64);
    }

    fun t2() {
        let S{ f, g }: S;
    }
}
