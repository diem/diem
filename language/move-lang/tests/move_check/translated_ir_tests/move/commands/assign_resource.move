module M {
    resource struct T {}

    fun no() {
        let t = T{}; &t;
        t = T {}; &t;
    }

}

// check: STLOC_UNSAFE_TO_DESTROY_ERROR
