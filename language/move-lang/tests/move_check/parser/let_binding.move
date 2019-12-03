module M {
    struct R {
        f: u64
    }
    struct Generic<T> {
        g: T
    }
    f() {
        let () = ();
        let (): () = ();
        // FIXME: The grammar currently treats "()" as a single token, but it is
        // not clear if that has any advantage compared to separate tokens that
        // allow whitespace in between them.
        // let ( ) = ( );
        // let ( ): ( ) = ( );
        let v1 = 1;
        let v2: u64 = 2;
        // FIXME: The grammar does not currently accept a single variable inside
        // parens, but for consistency, maybe that ought to be allowed.
        // let (v3) = 3;
        let (x1, x2) = (1, 2);
        let (x3, x4): (u64, u64) = (3, 4);
    }
    g(r: R, g: Generic<R>) {
        let R { f } = copy r;
        let (R { f: f1 }, R { f: f2 }) = (copy r, copy r);
        let Generic<R> { g: R { f: f3 } } = g;
    }
}
