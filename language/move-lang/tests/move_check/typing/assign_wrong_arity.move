module M {
    resource struct R {f: u64}

    fun t0() {
        let x;
        x = ();
        x = (0, 1, 2);
        () = 0;
        let b;
        let f;
        (x, b, R{f}) = (0, false, R{f: 0}, R{f: 0});
        (x, b, R{f}) = (0, false);
    }
}
