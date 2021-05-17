address 0x0 {
module A {

    struct S has key {
        x: u64
    }
    public fun read_at(addr: address): u64 acquires S {
        let s = borrow_global<S>(addr);
        s.x
    }
    spec read_at {
        pragma opaque = true;
        aborts_if !exists<S>(addr);
        ensures result == global<S>(addr).x;
    }

    public fun mutate_at(addr: address) acquires S {
        let s = borrow_global_mut<S>(addr);
        s.x = 2;
    }
    spec mutate_at {
        pragma opaque = true;
        ensures global<S>(addr).x == 2;
        aborts_if !exists<S>(addr);
        modifies global<S>(addr);
    }
}

module B {
    use 0x0::A;

    struct T has key {
        x: u64
    }

    public fun mutate_at_test_incorrect(addr1: address, addr2: address) acquires T {
        let x0 = A::read_at(addr2);
        let t = borrow_global_mut<T>(addr1);
        t.x = 2;
        let x1 = A::read_at(addr2);
        spec {
            assert x0 == x1;
        };
    }
    spec mutate_at_test_incorrect {
        pragma opaque = true;
        modifies global<T>(addr2);
    }

    public fun move_to_test_incorrect(account: &signer, addr2: address) {
        let x0 = A::read_at(addr2);
        move_to<T>(account, T{x: 2});
        let x1 = A::read_at(addr2);
        spec {
            assert x0 == x1;
        };
    }
    spec move_to_test_incorrect {
        pragma opaque = true;
        modifies global<T>(addr2);
    }

    public fun move_from_test_incorrect(addr1: address, addr2: address): T acquires T {
        let x0 = A::read_at(addr2);
        let v = move_from<T>(addr1);
        let x1 = A::read_at(addr2);
        spec {
            assert x0 == x1;
        };
        v
    }
    spec move_from_test_incorrect {
        pragma opaque = true;
        modifies global<T>(addr2);
    }

    public fun mutate_S_test1_incorrect(addr1: address, addr2: address) {
        let x0 = A::read_at(addr2);
        A::mutate_at(addr1);
        let x1 = A::read_at(addr2);
        spec {
            assert x0 == x1;
        };
    }
    spec mutate_S_test1_incorrect {
        requires addr1 != addr2;
        modifies global<A::S>(addr2);
    }

    public fun mutate_S_test2_incorrect(addr: address) {
        let x0 = A::read_at(addr);
        A::mutate_at(addr);
        let x1 = A::read_at(addr);
        spec {
            assert x0 == x1;
        };
    }
    spec mutate_S_test2_incorrect {
        modifies global<A::S>(addr);
    }
}
}
