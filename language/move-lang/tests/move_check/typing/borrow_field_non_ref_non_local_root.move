module M {
    struct S { f: u64 }

    fun t0(cond: bool, s: S) {
        (&foo().f: &u64);
        (&bar().f: &u64);
        (&mut bar().f: &mut u64);
        (&(if (cond) foo() else &bar()).f : &u64);
        (&(if (cond) *foo() else bar()).f : &u64);
    }

    fun foo(): &S {
        abort 0
    }

    fun bar(): S {
        S { f: 0 }
    }
}
