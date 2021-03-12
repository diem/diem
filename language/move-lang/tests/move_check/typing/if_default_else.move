module 0x8675309::M {
    fun t0(cond: bool) {
        if (cond) ();
        let () = if (cond) ();
        let () = if (cond) { let x = 0; x; };
    }
}
