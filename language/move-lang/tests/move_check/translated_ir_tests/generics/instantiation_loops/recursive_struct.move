address 0x42 {

module M0 {
    struct Foo { f: Foo }
}


module M1 {
    struct Cup<T> { f: T }
    struct Foo { f: Cup<Foo> }

    // blows up the stack
    public fun foo(): Foo {
        Foo { f: Cup<Foo> { f: foo() }}
    }

}

module M2 {
    resource struct X { y: vector<Y> }
    resource struct Y { x: vector<X> }

    // blows up the vm
    public fun ex(): bool {
        exists<X>(0x0::Transaction::sender())
    }

    // blows up the VM
    public fun borrow() acquires X {
        _ = borrow_global<X>(0x0::Transaction::sender())
    }
}

module M3 {
    use 0x42::M1;

    struct Foo { f: M1::Cup<Foo> }

}

module M3 {
    use 0x42::M1;

    struct A { b: B }
    struct B { c: C }
    struct C { d: vector<D> }
    struct D { x: M1::Cup<M1::Cup<M1::Cup<A>>> }

}

}
