address 0x42 {

module M0 {
    struct Foo { f: Foo }

    struct Cup<T> { f: T }
    struct Bar { f: Cup<Bar> }

    resource struct X { y: vector<Y> }
    resource struct Y { x: vector<X> }

}

module M1 {
    use 0x42::M0;

    struct Foo { f: M0::Cup<Foo> }

    struct A { b: B }
    struct B { c: C }
    struct C { d: vector<D> }
    struct D { x: M0::Cup<M0::Cup<M0::Cup<A>>> }
}

}
