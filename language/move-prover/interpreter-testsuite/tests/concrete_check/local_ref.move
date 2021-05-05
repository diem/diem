module 0x2::A {
    #[test]
    public fun local_imm_ref(): u64 {
        let a = 0;
        let b = &a;
        *b
    }

    #[test]
    public fun local_mut_ref(): u64 {
        let a = 0;
        let b = &mut a;
        *b = 1;
        a
    }
}
