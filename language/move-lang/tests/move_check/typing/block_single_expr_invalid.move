module M {
    resource struct R {}
    fun t0() {
        ({ 0 } : bool);
        ({ &0 } : u64);
        ({ &mut 0 } : ());
        ({ R {} } : R);
        ({ (0, false, false) } : (u64, bool));
    }
}
