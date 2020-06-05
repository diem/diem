address 0x2 {

module X {
    const C: u64 = 0;
}

module M {
    use 0x2::X::{Self, C};
    fun foo() {
        X::C;
        C;
    }
}

}
