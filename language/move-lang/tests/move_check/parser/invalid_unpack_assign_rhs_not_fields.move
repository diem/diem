address 0x1:

module X {}

module M {
    use 0x1::X;

    fun foo() {
        X::S () = 0;

        X::S 0 = 0;

        X::S { 0 } = 0;
    }
}
