module M {
    struct S { f: u64, g: bool }

    t0() {
        let x;
    }

    t1() {
        let (x, y);
    }

    t2() {
        let S{ f, g };
    }
}
