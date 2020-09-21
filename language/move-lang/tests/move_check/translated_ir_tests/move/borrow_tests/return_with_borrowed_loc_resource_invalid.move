module M {
    resource struct X { u: u64 }

    fun t() {
        let s = X { u: 0 };
        let u = &s.u;
        copy u;
    }
}

// check: RET_UNSAFE_TO_DESTROY_ERROR
