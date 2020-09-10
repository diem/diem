address 0x2 {

module X {}

module M {
    use 0x2::X;
    fun foo(x: X) {}
}

}
