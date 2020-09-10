address 0x2 {
module X {
    public fun u() {}
}

module M {
    use 0x2::X::{Self, u as X};
    fun foo() {
        X();
        X::u()
    }
}
}
