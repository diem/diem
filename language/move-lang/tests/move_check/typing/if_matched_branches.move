module M {
    resource struct R {}

    fun t0(cond: bool) {
        if (cond) () else ();
    }

    fun t1(cond: bool) {
        if (cond) 0x0 else 0x0;
        if (cond) false else false;
        R {} = if (cond) R{} else R{};
        if (cond) &0 else &1;
        if (cond) &mut 0 else &mut 1;
    }

    fun t2(cond: bool) {
        if (cond) (0, false) else (1, true);
        (_, _, _, R{}) = if (cond) (0, 0x0, &0, R{}) else (1, 0x1, &1, R{});
    }

}
