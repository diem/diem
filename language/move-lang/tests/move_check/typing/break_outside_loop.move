module 0x8675309::M {
    fun foo() {
        while (true) {
            break
        };
        break
    }

    fun bar() {
        break;
    }

    fun baz(x: u64): u64 {
        if (x >= 5) break;
        0
    }
}
