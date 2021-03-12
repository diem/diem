module 0x8675309::M {
    struct S has copy, drop { f: u64 }

    fun t0(cond: bool, _s: S) {
        (foo().f: u64);
        (bar().f: u64);
        ((if (cond) foo() else &bar()).f : u64);
        ((if (cond) *foo() else bar()).f : u64);
    }

    fun foo(): &S {
        abort 0
    }

    fun bar(): S {
        S { f: 0 }
    }
}
