module M {
    resource struct R {}
    t0() {
        ({ 0 } : u64);
        ({ &0 } : &u64);
        ({ &mut 0 } : &mut u64);
        R {} = ({ R {} } : R);
        ({ (0, false) } : (u64, bool));
    }
}
