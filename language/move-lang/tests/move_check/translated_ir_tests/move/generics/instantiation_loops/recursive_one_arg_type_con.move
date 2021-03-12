// Bad! Can have infinitely many instances: f<T>, f<S<T>>, f<S<S<T>>>, ...

module 0x8675309::M {
    struct S<T> { b: bool }

    fun f<T>(x: T) {
        f<S<T>>(S<T> { b: true })
    }
}
