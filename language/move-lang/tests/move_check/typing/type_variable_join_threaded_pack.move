address 0x1 {

module Container {
    struct T<V> {}

    public fun new<V>(): T<V> {
        T {}
    }

    public fun get<V: copyable>(_self: &T<V>): V {
        abort 0
    }

    public fun put<V>(_self: &mut T<V>, _item: V) {
        abort 0
    }
}


module M {
    use 0x1::Container;

    struct Box<T> { f1: T, f2: T }

    fun t0(): Box<u64> {
        let v = Container::new();
        let x = Container::get(&v);
        let b = Box { f1: x, f2: x };
        Container::put(&mut v, 0);
        b
    }
}

}
