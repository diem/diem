module M {
    fun t1() {
        let x = 0;
        let y = &x;
        x = 0;
        y;
        x;
    }

}

// check: STLOC_UNSAFE_TO_DESTROY_ERROR
