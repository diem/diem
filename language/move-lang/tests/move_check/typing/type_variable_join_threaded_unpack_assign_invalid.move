address 0x1:

module Container {
    struct T<V> {}

    public fun new<V>(): T<V> {
        T {}
    }

    public fun get<V: copyable>(self: &T<V>): V {
        abort 0
    }

    public fun put<V>(self: &mut T<V>, item: V) {
        abort 0
    }
}


module M {
    use 0x1::Container;

    struct Box<T> { f1: T, f2: T }
    resource struct R{}

    fun new<T>(): Box<T> {
        abort 0
    }

    fun t0(): bool {
        let v = Container::new();
        let f1;
        let f2;
        Box { f1, f2 }  = Container::get(&v);
        Container::put(&mut v, Box { f1: 0, f2: 0});
        f1
    }

    fun t1(): R {
        let v = Container::new();
        let f1;
        let f2;
        Box { f1, f2 }  = Container::get(&v);
        Container::put(&mut v, Box { f1: R{}, f2 });
        f1
    }
}
