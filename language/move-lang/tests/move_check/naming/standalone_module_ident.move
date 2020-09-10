address 0x2 {

module X {}

module M {
    use 0x2::X;
    fun foo() {
        let x = X;
        let x = 0x2::X;
        let y = 0x2::Y;
    }
}

}
