module 0x8675309::M {
    struct R { f:bool }
    fun t0() {
        let r = R{ f: false };
        let f;

        if (true) {
            R{ f } = move r;
        } else {
            R{ f } = move r;
            r = R{ f: false };
        };
        R{ f: _ } = move r;
        f;
    }
}
