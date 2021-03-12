module 0x8675309::M {
    struct T has drop {}
    struct S has drop { f: u64, g: u64 }
    fun foo() {
        let f = 0;
        let g = 1;
        T {};
        T { };
        S { f, g };
        S { f: 0, g: 0};
        S { g: 0, f };
        S { g, f: 0 };
    }
}
