address 0x2 {
module Exists {
    struct T has key, store {}

    struct S has key { f: address }

    struct V<phantom A> has key { }

    public fun exists_const(): bool {
        exists<T>(@0x1)
    }

    public fun exists_formal(a: address): bool {
        exists<T>(a)
    }

    public fun exists_field(s: &S): bool {
        exists<T>(s.f)
    }

    public fun exists_generic_instantiated(a: address): bool {
        exists<V<T>>(a)
    }

    public fun exists_generic<X: store>(a: address): bool {
        exists<V<X>>(a)
    }

    public fun call_with_type_param1(a: address): bool {
        exists_generic<T>(a)
    }

    public fun call_with_type_param2<X, Y: store>(a: address): bool {
        exists_generic<Y>(a)
    }

}
}
