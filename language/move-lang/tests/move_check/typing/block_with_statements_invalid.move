module M {
    resource struct R {}
    t0() {
        ({ let x = 0; x } : bool);
        ({ let x = 0; &x } : u64);
        ({ let y = 0; &mut (y + 1) } : ());
        ({ let r = { let r = R {}; r }; r } : R);
        ({ let x = 0; (x, false, false) } : (u64, bool));
    }
}
