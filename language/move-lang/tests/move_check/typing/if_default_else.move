module M {
    t0(cond: bool) {
        if (cond) ();
        let () = if (cond) ();
        let () = if (cond) { let x = 0; };
    }
}
