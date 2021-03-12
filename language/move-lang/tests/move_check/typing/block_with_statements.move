module 0x8675309::M {
    struct R {}
    fun t0() {
        ({ let x = 0; x } : u64);
        ({ let x = 0; &x } : &u64);
        ({ let y = 0; &mut (y + 1) } : &mut u64);
        R {} = ({ let r = { let r = R {}; r }; r } : R);
        ({ let x = 0; (x, false) } : (u64, bool));
    }
}
