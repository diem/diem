// Contains tests for treatment of opaque calls
module 0x42::Test {

	struct R has key { v: u64 }

	fun get_and_incr(addr: address): u64 acquires R {
	    if (!exists<R>(addr)) abort 33;
	    let r = borrow_global_mut<R>(addr);
	    let v = r.v;
	    r.v = r.v + 1;
	    v
	}
	spec get_and_incr {
	    pragma opaque;
	    requires addr != @0x0;
	    aborts_if !exists<R>(addr) with 33;
	    aborts_if global<R>(addr).v + 1 >= 18446744073709551615;
	    modifies global<R>(addr);
	    ensures global<R>(addr).v == old(global<R>(addr)).v + 1;
	    ensures result == global<R>(addr).v;
	}

	fun incr_twice() acquires R {
	    get_and_incr(@0x1);
	    get_and_incr(@0x1);
	}
	spec incr_twice {
	    aborts_if !exists<R>(@0x1) with 33;
	    ensures global<R>(@0x1).v == old(global<R>(@0x1)).v + 2;
	}
}
