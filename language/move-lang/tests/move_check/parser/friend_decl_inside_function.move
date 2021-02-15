address 0x42 {
module A {}

module M {
    fun M() {
        friend 0x42::A;
    }
}
}
