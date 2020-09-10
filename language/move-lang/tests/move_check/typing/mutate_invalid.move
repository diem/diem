module M {
    struct S { f: u64 }
    struct X { f: u64 }

    fun t0() {
        *&mut 0 = false;
        *&mut S{f:0}.f = &1;
        *foo(&mut 0) = (1, 0);
        bar(&mut S{f:0}).f = ();
        *&mut bar(&mut S{f:0}).f = &0;
        baz().f = false;
        *&mut baz().f = false;
    }

    fun t1() {
        let r = &mut S{ f: 0 };
        *r = X { f: 1 };

        r.f = &0;
        *&mut r.f = ();
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
