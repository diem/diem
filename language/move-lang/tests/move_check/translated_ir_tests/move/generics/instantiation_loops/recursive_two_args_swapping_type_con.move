// Similar to the case with one argument, but swaps the two type parameters.
// f<T1, T2> => f<S<T2>, T1> => f<S<T1>, S<T2>> => f<S<S<T2>>, S<T1>> => ...

module 0x8675309::M {
    struct S<T> { x: T }

    fun f<T1, T2>(a: T1, x: T2) {
        f<S<T2>, T1>(S<T2> { x }, a)
    }
}

// check: LOOP_IN_INSTANTIATION_GRAPH
