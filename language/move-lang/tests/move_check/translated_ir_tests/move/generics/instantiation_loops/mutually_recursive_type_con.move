// Not good: infinitely many types/instances.
//           f<T>, g<S<T>>, f<S<T>>, g<S<S<T>>>, ...

module M {
    struct S<T> { b: bool }

    fun f<T>() {
        g<S<T>>()
    }

    fun g<T>() {
        f<T>()
    }
}
