address 0x2 {
module M {
    fun t() {
        let x = 0;

        use 0x1::M::foo;
        foo(x)
    }
}
