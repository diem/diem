module M {
    struct S {}

    tmove() {
        let x: u64;
        let y = move x + 1;

        let s: S;
        let s2 = s;
    }

    tcopy() {
        let x: u64;
        let y = x + 1;

        let s: S;
        let s3 = copy s;
    }

    tborrow() {
        let x: u64;
        let y = &x;

        let s: S;
        let s2 = &s;
    }

}
