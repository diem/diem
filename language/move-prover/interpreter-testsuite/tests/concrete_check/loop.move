module 0x2::A {
    #[test]
    public fun loop_ind_var() {
        let i = 0;
        while (i < 10) {
            i = i + 1;
        };
    }

    #[test]
    public fun loop_ind_ref() {
        let i = 0;
        let p = &mut i;
        while (*p < 10) {
            *p = *p + 1;
        };
    }
}
