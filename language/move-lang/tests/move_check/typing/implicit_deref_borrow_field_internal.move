address 0x2 {

module X {
    struct S { f: u64 }
    public fun s(): S {
        S { f: 0 }
    }
}

module M {
    use 0x2::X;
    fun t0() {
        (X::s().f: u64);
        let s = &X::s();
        (s.f: u64);
        abort 0
    }
}

}
