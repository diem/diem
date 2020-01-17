module M {
    f(v: u64): u64 {
        if (v < 5) return 5;
	// Check for a useful diagnostic message when missing a space before "<"
        if (v< 10) return v;
        0
    }
}
