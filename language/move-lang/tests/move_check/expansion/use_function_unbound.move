address 0x2 {
module X {
}

module M {
    use 0x2::X::u;

    fun bar() {
        u()
    }
}
}
