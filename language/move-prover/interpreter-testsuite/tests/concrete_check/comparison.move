module 0x2::A {
    #[test]
    public fun compare(): (bool, bool, bool, bool, bool, bool) {
        let a = 1;
        let b = 2;
        (a < b, a <= b, a >= b, a > b, a == b, a != b)
    }
}
