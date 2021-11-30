module 0x42::Test {
    fun f(x: u64): u64 {
        x + x
    }
    spec f {
        ensures result == 2 * x;
    }
}
