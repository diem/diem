address 0x1 {
module X {
    public fun u() {}
}

module M {
    use 0x1::X::{Self, u as X};
    fun foo() {
        X();
        X::u()
    }
}
}
