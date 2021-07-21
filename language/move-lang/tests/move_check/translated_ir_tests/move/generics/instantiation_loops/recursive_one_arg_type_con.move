// Bad! Can have infinitely many instances: f<T>, f<S<T>>, f<S<S<T>>>, ...

module 0x8675309::M {
    struct S<T> { f: T }

    fun f<T>(x: T) {
        f<S<T>>(S<T> { f: x })
    }
}
