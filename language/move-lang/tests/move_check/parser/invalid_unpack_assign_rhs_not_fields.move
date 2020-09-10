address 0x2 {

module X {}

module M {
    use 0x2::X;

    fun foo() {
        X::S () = 0;

        X::S 0 = 0;

        X::S { 0 } = 0;
    }
}

}
