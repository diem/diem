module M {
    struct R {
        f: u64
    }
    struct Generic<T> {
        g: T
    }
    fun f() {
        let () = ();
        let (): () = ();
        // Test with whitespace between parenthesis.
        let ( ) = ( );
        let ( ): ( ) = ( );
        let v1 = 1;
        let v2: u64 = 2;
        let (v3) = 3; // for consistency, check a single variable inside parens
        let (x1, x2) = (1, 2);
        let (x3, x4): (u64, u64) = (3, 4);
        v1;
        v2;
        v3;
        x1;
        x2;
        x3;
        x4;
    }
    fun g(r: R, g: Generic<R>) {
        let R { f } = copy r;
        let (R { f: f1 }, R { f: f2 }) = (copy r, move r);
        let Generic<R> { g: R { f: f3 } } = g;
        f;
        f1;
        f2;
        f3;
    }
}
