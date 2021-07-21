module 0x8675309::M {
    struct S<T> { f: T }

    fun a<T>() {
        b<S<T>, u64>()
    }

    fun b<T1, T2>() {
        f<T1>();
        c<S<T2>, bool>()
    }

    fun c<T1, T2>() {
        c<u64, T1>();
        d<T2>();
        e<T2>()
    }

    fun d<T>() {
        b<u64, T>()
    }

    fun e<T>() {
    }

    fun f<T>() {
        g<T>()
    }

    fun g<T>() {
        f<S<T>>()
    }
}

// There are two disjoint loops in the resulting graph:

// loop 1: f#T --> g#T --S<T>--> f#T

// loop 2: c#T1 --> c#T2 --> d#T --> b#T2 --S<T2>--> c#T1
