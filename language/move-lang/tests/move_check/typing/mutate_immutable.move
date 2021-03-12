module 0x8675309::M {
    struct S has drop { f: u64 }

    fun t0(s: &mut S) {
        *(s: &S) = S { f: 0 };
        *&0 = 1;
        let x = 0;
        let x_ref = &mut x;
        let x_ref: &u64 = x_ref;
        *x_ref = 0;
    }

}
