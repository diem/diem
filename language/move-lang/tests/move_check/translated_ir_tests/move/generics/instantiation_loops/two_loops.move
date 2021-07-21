// Two loops in the resulting graph.
// One error for each loop.

module 0x8675309::M {
    struct S<T> { f: T }

    fun f<T>() {
        f<S<T>>()
    }

    fun g<T>() {
        g<S<T>>()
    }
}

// check: LOOP_IN_INSTANTIATION_GRAPH
// check: LOOP_IN_INSTANTIATION_GRAPH
