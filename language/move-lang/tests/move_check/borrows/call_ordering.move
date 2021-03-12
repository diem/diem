module 0x8675309::M {
    struct S { f: u64 }

    fun foo(_s: &S, _u: u64) {}
    fun t0(s: &mut S) {
        let f = &mut s.f;
        foo(freeze(s), { *f = 0; 1 })
    }

    fun bar(_s: &mut u64, _u: u64) {}
    fun t1(s: &mut S) {
        bar(&mut s.f, { s.f = 0; 1 })
    }
}
