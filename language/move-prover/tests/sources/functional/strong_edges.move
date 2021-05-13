// This file consists of a series of test cases which are client functions
// using the standard vector module.
module 0x42::TestStrongEdges {
    use Std::Vector;

    spec module {
        pragma verify = true;
    }

    struct S has key {
        x: u64
    }

    public fun glob_and_field_edges(addr: address) acquires S {
        let s = borrow_global_mut<S>(addr);
        s.x = 2;
    }

    spec glob_and_field_edges{
        pragma opaque = true;
        ensures global<S>(addr).x == 2;
        aborts_if !exists<S>(addr);
        modifies global<S>(addr);
    }

    fun loc_edge() {
        let r = 5;
        let r_ref = &mut r;
        *r_ref = 6;
        spec {
            assert r == 6;
        };
    }

    fun vec_edge(v: &mut vector<u64>) : u64
    {
        let x = *Vector::borrow(v, 0);
        *Vector::borrow_mut(v, 0) = 7;
        x
    }
    spec vec_edge {
        aborts_if len(v) == 0;
        ensures v[0] == 7;
        ensures v[1] == old(v[1]);
    }

    public fun glob_and_field_edges_incorrect(addr: address) acquires S {
        let s = borrow_global_mut<S>(addr);
        s.x = 2;
    }

    spec glob_and_field_edges_incorrect{
        pragma opaque = true;
        ensures global<S>(addr).x == 3;
        aborts_if !exists<S>(addr);
        modifies global<S>(addr);
    }

    fun loc__edge_incorrect() {
        let r = 5;
        let r_ref = &mut r;
        *r_ref = 6;
        spec {
            assert r == 5;
        };
    }
}
