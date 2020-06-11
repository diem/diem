address 0x2 {

module X {
    public fun foo(): u64 { 0 }
}

module M {
    use 0x2::X;

    fun foo(): u64 { 0 }

    fun t0() {
        Self::fooo();
        foooo();
        X::foooooo();
    }
}

}
