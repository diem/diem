module 0x2::A {
    #[test]
    public fun if_else(): (u64, u64) {
        let a = 1;
        let b = 2;
        let r = if (a > b) { &mut a } else  { &mut b };
        *r = 3;
        (a, b)
    }
}
