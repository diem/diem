module Test {
	fun limited_div(a: u64, b: u64): u64 {
	    a / b
	}
	spec fun limited_div {
	    requires a < 100;
	    aborts_if b == 0;
	    aborts_if a == 0;
	    ensures result == a / b;
	}

	resource struct R { v: u64 }

	fun increment_R(val: u64) acquires R {
	    let r = borrow_global_mut<R>(0x0);
	    r.v = r.v + val;
	}
	spec fun increment_R {
	    requires val > 0;
	    aborts_if !exists<R>(0x0);
	    aborts_if global<R>(0x0).v + val >= 18446744073709551615;
	    ensures global<R>(0x0).v == old(global<R>(0x0)).v + val;
	}
}
