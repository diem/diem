module 0x2::A {
    // TODO (mengxu): this is actually tricky, the two `i` appear in different temporaries and
    // there is no linkage between these temporaries (except for the appearance in trace_local).
    public fun loop_ind_var() {
        let i = 0;
        while (i < 10) {
            i = i + 1;
        };
    }

    // TODO (mengxu): there is an error in the transformation pipeline, a destroy($t) appears before the writeback($t).
    public fun loop_ind_ref() {
        let i = 0;
        let p = &mut i;
        while (*p < 10) {
            *p = *p + 1;
        };
    }
}
