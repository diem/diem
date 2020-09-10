address 0x0 {
module A {
    resource struct S {
        x: u64
    }

    public fun read_at(addr: address): u64 acquires S {
        let s = borrow_global<S>(addr);
        s.x
    }
    spec fun read_at {
        pragma opaque = true;
        aborts_if !exists<S>(addr);
        ensures result == global<S>(addr).x;
    }

    public fun mutate_at(addr: address) acquires S {
        let s = borrow_global_mut<S>(addr);
        s.x = 2;
    }
    spec fun mutate_at {
        pragma opaque = true;
        ensures global<S>(addr).x == 2;
        aborts_if !exists<S>(addr);
        modifies global<S>(addr);
    }
}

module B {
    use 0x0::A;
    use 0x1::Signer;

    resource struct T {
        x: u64
    }

    public fun mutate_at_test(addr: address) acquires T {
        let t = borrow_global_mut<T>(addr);
        t.x = 2;
    }
    spec fun mutate_at_test {
        pragma opaque = true;
        ensures global<T>(addr).x == 2;
        modifies global<T>(addr);
    }

    public fun move_to_test(account: &signer) {
        move_to<T>(account, T{x: 2});
    }
    spec fun move_to_test {
        pragma opaque = true;
        ensures global<T>(Signer::spec_address_of(account)).x == 2;
        modifies global<T>(Signer::spec_address_of(account));
    }

    public fun move_from_test(addr: address): T acquires T {
        move_from<T>(addr)
    }
    spec fun move_from_test {
        pragma opaque = true;
        requires global<T>(addr).x == 2;
        ensures result.x == 2;
        modifies global<T>(addr);
    }

    public fun mutate_S_test(addr1: address, addr2: address) acquires T {
        let x0 = A::read_at(addr2);
        A::mutate_at(addr1);
        let x1 = A::read_at(addr2);
        spec {
            assert x0 == x1;
        };
        mutate_at_test(addr2)
    }
    spec fun mutate_S_test {
        requires addr1 != addr2;
        ensures global<A::S>(addr2) == old(global<A::S>(addr2));
        ensures global<A::S>(addr1).x == 2;
        ensures global<T>(addr2).x == 2;
        modifies global<A::S>(addr1), global<T>(addr2);
    }
}
}
