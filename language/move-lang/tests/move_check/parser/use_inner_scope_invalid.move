address 0x1 {
module M {
    fun t() {
        let x = 0;

        use 0x0::M::foo;
        foo(x)
    }
}
