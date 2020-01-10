address 0x1:

module Container {
    struct T<V> {}

    public new<V>(): T<V> {
        T {}
    }

    public get<V: copyable>(self: &T<V>): V {
        abort 0
    }

    public put<V>(self: &mut T<V>, item: V) {
        abort 0
    }
}


module M {
    use 0x1::Container;

    struct Box<T> { f1: T, f2: T }

    new<T>(): Box<T> {
        abort 0
    }

    t0(): u64 {
        let v = Container::new();
        let f1;
        let f2;
        Box { f1, f2 }  = Container::get(&v);
        f2;
        Container::put(&mut v, Box { f1: 0, f2: 0});
        f1
    }

    t1(): Box<Box<u64>> {
        let v = Container::new();
        let f1;
        let f2;
        Box { f1, f2 }  = Container::get(&v);
        Container::put(&mut v, Box { f1: *&f1, f2 });
        f1
    }
}
