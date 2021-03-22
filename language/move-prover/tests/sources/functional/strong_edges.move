// flag: --strong_edges
// This file consists of a series of test cases which are client functions
// using the standard vector module.
module TestStrongEdges {
    use 0x1::Vector;

    spec module {
        pragma verify = true;
    }

    struct S has key {
        x: u64
    }

    public fun glob_and_field(addr: address) acquires S {
        let s = borrow_global_mut<S>(addr);
        s.x = 2;
    }

    spec fun glob_and_field{
        pragma opaque = true;
        ensures global<S>(addr).x == 2;
        aborts_if !exists<S>(addr);
        modifies global<S>(addr);
    }

    fun loc() {
        let r = 5;
        let r_ref = &mut r;
        *r_ref = 6;
        spec {
            assert r == 6;
        };
    }

    fun test_borrow_mut(v: &mut vector<u64>) : u64
    {
        let x = *Vector::borrow(v, 0);
        *Vector::borrow_mut(v, 0) = 7;
        x
    }
    spec fun test_borrow_mut {
        aborts_if len(v) == 0;
        ensures v[0] == 7;
        ensures v[1] == old(v[1]);
    }

    public fun glob_and_field_incorrect(addr: address) acquires S {
        let s = borrow_global_mut<S>(addr);
        s.x = 2;
    }

    spec fun glob_and_field_incorrect{
        pragma opaque = true;
        ensures global<S>(addr).x == 3;
        aborts_if !exists<S>(addr);
        modifies global<S>(addr);
    }

    fun loc_incorrect() {
        let r = 5;
        let r_ref = &mut r;
        *r_ref = 6;
        spec {
            assert r == 5;
        };
    }

}
