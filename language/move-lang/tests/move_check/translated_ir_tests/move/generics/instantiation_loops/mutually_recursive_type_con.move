// Not good: infinitely many types/instances.
//           f<T>, g<S<T>>, f<S<T>>, g<S<S<T>>>, ...

module 0x8675309::M {
    struct S<T> { f: T }

    fun f<T>() {
        g<S<T>>()
    }

    fun g<T>() {
        f<T>()
    }
}
