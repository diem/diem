module 0x2::A {
    #[test]
    public fun bitand(): u8 {
        let a = 0x1 & 0x2;
        a
    }

    #[test]
    public fun bitor(): u8 {
        let a = 0x1 | 0x2;
        a
    }

    #[test]
    public fun bitxor(): u8 {
        let a = 0x1 ^ 0x2;
        a
    }

    #[test]
    public fun shl(): u8 {
        let a = 0x1 << 2;
        a
    }

    #[test]
    public fun shr(): u8 {
        let a = 0x10 >> 2;
        a
    }
}
