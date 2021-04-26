address 0x2 {
module X {
    public fun u(): u64 {
        0
    }
    public fun a(): address {
        @0x0
    }
    public fun bar() {}
}

module M {
    use 0x2::X::{u, u as U, u as get_u, a, a as A, a as get_a};
    fun t() {
        (u(): u64);
        (U(): u64);
        (get_u(): u64);
        (a(): address);
        (A(): address);
        (get_a(): address);
    }

    use 0x2::X::bar as baz;
    fun bar() {
        baz()
    }
}

}
