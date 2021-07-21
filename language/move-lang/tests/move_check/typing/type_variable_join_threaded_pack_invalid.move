address 0x2 {

module Container {
    struct T<V> { f: V }

    public fun new<V>(): T<V> {
        abort 0
    }

    public fun get<V: drop>(self: &T<V>): V {
        abort 0
    }

    public fun put<V>(self: &mut T<V>, item: V) {
        abort 0
    }

    public fun get_ref<V: drop>(self: &T<V>): &V {
        abort 0
    }
}


module M {
    use 0x2::Container;

    struct Box<T> { f1: T, f2: T }
    struct R {}

    fun id<T>(r: &T): &T {
        r
    }


    fun t0(): Box<bool> {
        let v = Container::new();
        let x = Container::get(&v);
        let b = Box { f1: x, f2: x };
        Container::put(&mut v, 0);
        let r = Container::get_ref(&v);
        id(r);
        b
    }

    fun t2(): Box<Box<R>> {
        let v = Container::new();
        let x = Container::get(&v);
        let b = Box { f1: x, f2: x };
        Container::put(&mut v, Box {f1: R{}, f2: R{}});
        b
    }
}

}
