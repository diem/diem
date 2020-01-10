module M {
    struct S {}

    tmove() {
        let x: u64;
        let _ = move x + 1;

        let s: S;
        let _s2 = s;
    }

    tcopy() {
        let x: u64;
        let _ = x + 1;

        let s: S;
        let _s3 = copy s;
    }

    tborrow() {
        let x: u64;
        let _ = &x;

        let s: S;
        let _s2 = &s;
    }

}
