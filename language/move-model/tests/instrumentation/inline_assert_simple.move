module 0x42::M {
    fun foo() {
        let x: u64 = 0;
        spec {
            assert x == 0;
        };
    }
}
