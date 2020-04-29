address 0x1 {

module X {
    struct S { f: u64 }
    public fun s(): S {
        S { f: 0 }
    }
}

module M {
    use 0x1::X;
    fun t0() {
        (X::s().f: u64);
        let s = &X::s();
        (s.f: u64);
    }
}

}
