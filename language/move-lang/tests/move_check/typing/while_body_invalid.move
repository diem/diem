module 0x8675309::M {
    fun t0(cond: bool) {
        while (cond) 0;
        while (cond) false;
        while (cond) { @0x0 };
        while (cond) { let x = 0; x };
        while (cond) { if (cond) 1 else 0 };
    }
}
