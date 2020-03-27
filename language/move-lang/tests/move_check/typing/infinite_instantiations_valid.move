address 0x42:

module M {
    struct Box<T> {}

    public fun t0<T>() {
        t1<T>();
        t0<T>()
    }

    public fun t1<T>() {
        t0<T>();
        t1<T>()
    }

    public fun x<T>() {
        y<Box<T>>()
    }
    public fun y<T>() {
        z<Box<T>>()
    }
    public fun z<T>() {
        z<T>()
    }

}

module N {
    use 0x42::M;
    public fun t<T>() {
        M::t0<M::Box<T>>()
    }
}
