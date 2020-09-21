// Two loops in the resulting graph.
// One error for each loop.

module M {
    struct S<T> { b: bool }

    fun f<T>() {
        f<S<T>>()
    }

    fun g<T>() {
        g<S<T>>()
    }
}

// check: LOOP_IN_INSTANTIATION_GRAPH
// check: LOOP_IN_INSTANTIATION_GRAPH
