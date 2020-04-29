address 0x1 {

module X {
    public fun foo(): u64 { 0 }
}

module M {
    use 0x1::X;

    fun foo(): u64 { 0 }

    fun t0() {
        Self::fooo();
        foooo();
        X::foooooo();
    }
}

}
