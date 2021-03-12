module 0x8675309::M {
    struct S { f: u64 }
    struct R {}

    fun t0(u: u64, cond: bool, addr: address) {
        &0.f;
        &0.g;
        &u.value;
        &cond.value;
        &addr.R;
        &addr.f;
        &().R;
        &(&S{f: 0}, &S{f:0}).f;
    }
}
