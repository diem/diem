module M {
    struct S { f: u64 }
    fun t0() {
        *&mut 0 = 1;
        *&mut S{f:0}.f = 1;
        *foo(&mut 0) = 1;
        bar(&mut S{f:0}).f = 1;
        *&mut bar(&mut S{f:0}).f = 1;
        baz().f = 1;
        *&mut baz().f = 1;
    }

    fun t1() {
        let r = &mut S{ f: 0 };
        *r = S { f: 1 };

        r.f = 1;
        *&mut r.f = 1;
    }

    fun foo(x: &mut u64): &mut u64 {
        x
    }

    fun bar(s: &mut S): &mut S {
        s
    }

    fun baz(): S {
        S { f: 0 }
    }
}
