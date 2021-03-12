module 0x8675309::M {
    struct R {}
    fun t0() {
        ({ 0 } : u64);
        ({ &0 } : &u64);
        ({ &mut 0 } : &mut u64);
        R {} = ({ R {} } : R);
        ({ (0, false) } : (u64, bool));
    }
}
