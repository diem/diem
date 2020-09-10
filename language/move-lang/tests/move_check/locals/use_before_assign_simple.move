module M {
    struct S {}

    fun tmove() {
        let x: u64;
        let _ = move x + 1;

        let s: S;
        let _s2 = s;
    }

    fun tcopy() {
        let x: u64;
        let _ = x + 1;

        let s: S;
        let _s3 = copy s;
    }

    fun tborrow() {
        let x: u64;
        let _ = &x;

        let s: S;
        let _s2 = &s;
    }

}
