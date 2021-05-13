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
    use Std::Signer;

    struct X has key, store { y: vector<Y> }
    struct Y has key, store { x: vector<X> }

    // blows up the vm
    public fun ex(account: &signer): bool {
        let sender = Signer::address_of(account);
        exists<X>(sender)
    }

    // blows up the VM
    public fun borrow(account: &signer) acquires X {
        let sender = Signer::address_of(account);
        _ = borrow_global<X>(sender)
    }
}

module M3 {
    use 0x42::M1;

    struct Foo { f: M1::Cup<Foo> }

}

module M4 {
    use 0x42::M1;

    struct A { b: B }
    struct B { c: C }
    struct C { d: vector<D> }
    struct D { x: M1::Cup<M1::Cup<M1::Cup<A>>> }

}

}
