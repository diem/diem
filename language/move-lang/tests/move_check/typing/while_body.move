module 0x8675309::M {
    fun t0(cond: bool) {
        while (cond) ();
        while (cond) (());
        while (cond) {};
        while (cond) { let x = 0; x; };
        while (cond) { if (cond) () };
        while (cond) break;
        while (cond) { break };
        while (cond) continue;
        while (cond) { continue };
        while (cond) return ();
        while (cond) { while (cond) { break } }
    }
}
