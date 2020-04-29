address 0x1 {

module X {}

module M {
    use 0x1::X;
    fun foo() {
        let x = X;
        let x = 0x1::X;
        let y = 0x1::Y;
    }
}

}
