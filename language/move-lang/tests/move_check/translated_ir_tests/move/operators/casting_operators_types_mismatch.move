module 0x8675309::M {
    struct S { x: bool }
    fun f() {
        (S { x: true } as u8);
        (S { x: true } as u64);
        (S { x: true } as u128);
        (true as u8);
        (true as u64);
        (true as u128);
        (@0x0 as u64);
        (@0x0 as u128);
    }
}
